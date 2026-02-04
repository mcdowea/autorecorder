/*
 * =====================================================================================
 *
 * Filename:  WeChatRecorder.cpp
 *
 * Description:  Main application logic with final UI polish.
 *
 * =====================================================================================
 */
#include "framework.h"
#include <windows.h>
#include <atomic>
#include <algorithm>
#include <filesystem>
#include <vector>
#include <string>
#include <shellapi.h> 

#include "resource.h"
#include "config_manager.h"
#include "wasapi_recorder.h"
#include "wechat_detector.h"
#include "device_page.h"
#include "path_page.h"
#include "about_page.h"
#include "general_page.h"
#include "blacklist_page.h" 
 // #include "custom_button.h" // No longer needed
#include "path_util.h"
#include "log.h"
#include "WeChatRecorder.h"

#define WM_APP_TRAY_MSG (WM_APP + 1)

// --- Global Variables ---
HINSTANCE hInst;
HWND hMainDialog = NULL;
HWND hGeneralPage = NULL, hDevicePage = NULL, hPathPage = NULL, hAboutPage = NULL, hBlacklistPage = NULL;
WasapiRecorder g_recorder;
HANDLE hMonitorThread = NULL;
std::atomic<bool> monitorRunning = false;
bool isRecording = false;
bool isMonitorStarted = false;
bool isManualRecording = false;  // 手动录音标志
std::wstring g_lastMicAppName;
std::wstring g_currentRecordingPath;  // 当前录音文件路径
std::vector<std::wstring> g_blacklist;

#define TIMER_UPDATE_UI 1  // UI更新定时器ID

// --- Forward Declarations ---
void AddTrayIcon(HWND hWnd);
void RemoveTrayIcon(HWND hWnd);
void ShowTrayMenu(HWND hWnd);
void UpdateTrayTip(HWND hWnd, const std::wstring& tipText);
void UpdateRecordingInfo(HWND hWnd);  // 更新录音信息显示
void OpenRecordingFolder();  // 打开录音文件夹
void StartManualRecording(HWND hWnd);  // 开始手动录音
void StopManualRecording(HWND hWnd);  // 停止手动录音
INT_PTR CALLBACK SettingsDlgProc(HWND hDlg, UINT message, WPARAM wParam, LPARAM lParam);
INT_PTR CALLBACK MainDialogProc(HWND hDlg, UINT message, WPARAM wParam, LPARAM lParam);


void StartRecording(HWND hWnd) {
    if (isRecording) return;
    std::wstring appName;
    if (!IsMicInUse(appName)) return;
    g_lastMicAppName = appName;

    auto it = std::find_if(g_blacklist.begin(), g_blacklist.end(), [&](const std::wstring& blockedApp) {
        return _wcsicmp(blockedApp.c_str(), appName.c_str()) == 0;
        });

    if (it != g_blacklist.end()) {
        WriteLog(L"[主线程] 应用 %s 在黑名单中，已阻止录音。", appName.c_str());
        SetDlgItemText(hWnd, IDC_LABEL_STATUS, (L"已阻止: " + appName).c_str());
        return;
    }

    std::wstring inputDeviceId = GetSelectedInputDeviceId();
    std::wstring outputDeviceId = GetSelectedOutputDeviceId();

    if (inputDeviceId.empty() || outputDeviceId.empty()) {
        WriteLog(L"[错误] 启动录音失败：未选择有效的输入或输出设备。");
        MessageBoxW(hWnd, L"请在“设置”中选择有效的输入和输出设备。", L"错误", MB_OK | MB_ICONERROR);
        return;
    }

    std::wstring exeName = appName;
    size_t pos = exeName.rfind(L".exe");
    if (pos != std::wstring::npos && pos == exeName.length() - 4) {
        exeName = exeName.substr(0, pos);
    }

    std::wstring filePath = GenerateSavePath(g_savePath, exeName);
    std::filesystem::create_directories(std::filesystem::path(filePath).parent_path());

    if (g_recorder.Start(filePath, inputDeviceId, outputDeviceId, g_micVolumePercent, g_speakerVolumePercent)) {
        isRecording = true;
        g_currentRecordingPath = filePath;  // 保存当前录音路径
        std::wstring statusText = L"正在录制: " + appName;
        SetDlgItemText(hWnd, IDC_LABEL_STATUS, statusText.c_str());
        UpdateTrayTip(hWnd, statusText);
        
        // 启动UI更新定时器
        SetTimer(hWnd, TIMER_UPDATE_UI, 500, NULL);  // 每500ms更新一次
    }
    else {
        isRecording = false;
        SetDlgItemText(hWnd, IDC_LABEL_STATUS, L"启动失败");
        UpdateTrayTip(hWnd, L"启动失败");
    }
}

void StopRecording(HWND hWnd) {
    if (!isRecording) return;
    g_recorder.Stop();
    isRecording = false;
    isManualRecording = false;
    
    // 停止UI更新定时器
    KillTimer(hWnd, TIMER_UPDATE_UI);

    std::wstring statusText = isMonitorStarted ? L"已启动检测..." : L"等待中...";
    SetDlgItemText(hWnd, IDC_LABEL_STATUS, statusText.c_str());
    SetDlgItemText(hWnd, IDC_LABEL_DURATION, L"录音时长: 00:00:00");
    SetDlgItemText(hWnd, IDC_LABEL_FILESIZE, L"文件大小: 0 KB");
    UpdateTrayTip(hWnd, statusText);
}

DWORD WINAPI MonitorThreadProc(LPVOID lpParam) {
    HWND hWnd = (HWND)lpParam;
    while (monitorRunning) {
        std::wstring appName;
        if (IsMicInUse(appName)) {
            if (!isRecording) PostMessage(hWnd, WM_USER + 1, 0, 0);
        }
        else {
            if (isRecording) PostMessage(hWnd, WM_USER + 2, 0, 0);
        }
        Sleep(1000);
    }
    return 0;
}

INT_PTR CALLBACK SettingsDlgProc(HWND hDlg, UINT message, WPARAM wParam, LPARAM lParam) {
    switch (message) {
    case WM_INITDIALOG: {
        // *** FIX: Reverted to using the placeholder control for robust positioning ***
        HWND hPlaceholder = GetDlgItem(hDlg, IDC_SUBPAGE_PLACEHOLDER);
        RECT rc;
        GetWindowRect(hPlaceholder, &rc);
        MapWindowPoints(HWND_DESKTOP, hDlg, (LPPOINT)&rc, 2);
        DestroyWindow(hPlaceholder);

        hGeneralPage = CreateDialog(hInst, MAKEINTRESOURCE(IDD_PAGE_GENERAL), hDlg, GeneralPageProc);
        hDevicePage = CreateDialog(hInst, MAKEINTRESOURCE(IDD_PAGE_DEVICE), hDlg, DevicePageProc);
        hPathPage = CreateDialog(hInst, MAKEINTRESOURCE(IDD_PAGE_PATH), hDlg, PathPageProc);
        hBlacklistPage = CreateDialog(hInst, MAKEINTRESOURCE(IDD_PAGE_BLACKLIST), hDlg, BlacklistPageProc);
        hAboutPage = CreateDialog(hInst, MAKEINTRESOURCE(IDD_PAGE_ABOUT), hDlg, AboutPageProc);

        HWND pages[] = { hGeneralPage, hDevicePage, hPathPage, hBlacklistPage, hAboutPage };

        for (HWND page : pages) {
            if (page) {
                // Use the placeholder's rectangle to position all sub-pages
                MoveWindow(page, rc.left, rc.top, rc.right - rc.left, rc.bottom - rc.top, TRUE);
                ShowWindow(page, SW_HIDE);
            }
        }
        if (hGeneralPage) ShowWindow(hGeneralPage, SW_SHOW);
        return TRUE;
    }
    case WM_COMMAND: {
        HWND pagesToShow[] = { hGeneralPage, hDevicePage, hPathPage, hBlacklistPage, hAboutPage };
        for (HWND page : pagesToShow) if (page) ShowWindow(page, SW_HIDE);

        HWND pageToShow = NULL;
        switch (LOWORD(wParam)) {
        case IDC_BTN_GENERAL:   pageToShow = hGeneralPage;    break;
        case IDC_BTN_DEVICE:    pageToShow = hDevicePage;     break;
        case IDC_BTN_PATH:      pageToShow = hPathPage;       break;
        case IDC_BTN_BLACKLIST: pageToShow = hBlacklistPage;  break;
        case IDC_BTN_ABOUT:     pageToShow = hAboutPage;      break;
        case IDOK: case IDCANCEL:
            EndDialog(hDlg, 0);
            return TRUE;
        }
        if (pageToShow) ShowWindow(pageToShow, SW_SHOW);
        return TRUE;
    }
    case WM_DESTROY:
        SaveConfig();
        HWND pagesToDestroy[] = { hGeneralPage, hDevicePage, hPathPage, hBlacklistPage, hAboutPage };
        for (HWND page : pagesToDestroy) if (page) DestroyWindow(page);
        return TRUE;
    }
    return FALSE;
}

INT_PTR CALLBACK MainDialogProc(HWND hDlg, UINT message, WPARAM wParam, LPARAM lParam) {
    hMainDialog = hDlg;
    switch (message) {
    case WM_INITDIALOG: {
        InitializeDefaultDevices();
        HICON hIcon = LoadIcon(hInst, MAKEINTRESOURCE(IDI_WECHATRECORDER));
        SendMessage(hDlg, WM_SETICON, ICON_BIG, (LPARAM)hIcon);
        HICON hIconSmall = LoadIcon(hInst, MAKEINTRESOURCE(IDI_SMALL));
        SendMessage(hDlg, WM_SETICON, ICON_SMALL, (LPARAM)hIconSmall);

        RECT rcDlg; GetWindowRect(hDlg, &rcDlg);
        int dlgW = rcDlg.right - rcDlg.left, dlgH = rcDlg.bottom - rcDlg.top;
        int screenW = GetSystemMetrics(SM_CXSCREEN), screenH = GetSystemMetrics(SM_CYSCREEN);
        SetWindowPos(hDlg, NULL, (screenW - dlgW) / 2, (screenH - dlgH) / 2, 0, 0, SWP_NOSIZE | SWP_NOZORDER);

        SetDlgItemText(hDlg, IDC_LABEL_STATUS, L"等待中...");
        SetDlgItemText(hDlg, IDC_LABEL_DURATION, L"录音时长: 00:00:00");
        SetDlgItemText(hDlg, IDC_LABEL_FILESIZE, L"文件大小: 0 KB");
        UpdateTrayTip(hDlg, L"等待中...");
        return TRUE;
    }
    case WM_APP_TRAY_MSG: {
        switch (lParam) {
        case WM_LBUTTONDBLCLK:
            ShowWindow(hDlg, SW_RESTORE);
            SetForegroundWindow(hDlg);
            RemoveTrayIcon(hDlg);
            break;
        case WM_RBUTTONUP:
            ShowTrayMenu(hDlg);
            break;
        }
        return TRUE;
    }
    case WM_SYSCOMMAND: {
        if (wParam == SC_CLOSE) {
            if (g_minimizeToTray) {
                AddTrayIcon(hDlg);
                ShowWindow(hDlg, SW_HIDE);
            }
            else {
                SendMessage(hDlg, WM_CLOSE, 0, 0);
            }
            return TRUE;
        }
        break;
    }
    case WM_USER + 1: StartRecording(hDlg); return TRUE;
    case WM_USER + 2: StopRecording(hDlg); return TRUE;
    case WM_COMMAND:
        switch (LOWORD(wParam)) {
        case IDC_BTN_START:
            if (!isMonitorStarted) {
                isMonitorStarted = true; monitorRunning = true;
                hMonitorThread = CreateThread(NULL, 0, MonitorThreadProc, hDlg, 0, NULL);
                SetDlgItemText(hDlg, IDC_LABEL_STATUS, L"已启动检测...");
                SetDlgItemText(hDlg, IDC_BTN_START, L"停止检测");
                UpdateTrayTip(hDlg, L"已启动检测...");
            }
            else {
                monitorRunning = false;
                if (hMonitorThread) { WaitForSingleObject(hMonitorThread, 1500); CloseHandle(hMonitorThread); hMonitorThread = NULL; }
                if (isRecording) StopRecording(hDlg);
                isMonitorStarted = false;
                SetDlgItemText(hDlg, IDC_LABEL_STATUS, L"等待中...");
                SetDlgItemText(hDlg, IDC_BTN_START, L"开始检测");
                UpdateTrayTip(hDlg, L"等待中...");
            }
            break;
        case IDC_BTN_SETTINGS:
            DialogBox(hInst, MAKEINTRESOURCE(IDD_SETTINGS_DIALOG), hDlg, SettingsDlgProc);
            break;
        case IDC_BTN_OPEN_FOLDER:
            OpenRecordingFolder();
            break;
        case IDC_BTN_MANUAL_RECORD:
            if (!isManualRecording) {
                StartManualRecording(hDlg);
            } else {
                StopManualRecording(hDlg);
            }
            break;
        case ID_TRAY_SHOW:
            ShowWindow(hDlg, SW_RESTORE);
            SetForegroundWindow(hDlg);
            RemoveTrayIcon(hDlg);
            break;
        case ID_TRAY_EXIT:
            SendMessage(hDlg, WM_CLOSE, 0, 0);
            break;
        }
        return TRUE;
    case WM_TIMER:
        if (wParam == TIMER_UPDATE_UI) {
            UpdateRecordingInfo(hDlg);
        }
        return TRUE;
    case WM_CLOSE:
        monitorRunning = false;
        KillTimer(hDlg, TIMER_UPDATE_UI);  // 清理定时器
        if (hMonitorThread) { WaitForSingleObject(hMonitorThread, 1000); CloseHandle(hMonitorThread); }
        if (isRecording) StopRecording(hDlg);
        RemoveTrayIcon(hDlg);
        SaveConfig();
        EndDialog(hDlg, 0);
        return TRUE;
    case WM_DESTROY:
        PostQuitMessage(0);
        return TRUE;
    }
    return FALSE;
}

int APIENTRY wWinMain(HINSTANCE hInstance, HINSTANCE hPrevInstance, LPWSTR lpCmdLine, int nCmdShow) {
    const wchar_t* appName = L"自动拾音 --by夜莺";
    const wchar_t* mutexName = L"{8F4A3A6E-4556-4B18-831E-164741A6C5F3}-自动拾音 --by夜莺";
    HANDLE hMutex = CreateMutex(NULL, TRUE, mutexName);

    if (hMutex != NULL && GetLastError() == ERROR_ALREADY_EXISTS) {
        HWND hWndExisting = FindWindow(NULL, appName);
        if (hWndExisting) {
            ShowWindow(hWndExisting, SW_RESTORE);
            SetForegroundWindow(hWndExisting);
        }
        CloseHandle(hMutex);
        return 1;
    }

    hInst = hInstance;
    LoadConfig();
    DialogBox(hInstance, MAKEINTRESOURCE(IDD_MAIN_DIALOG), NULL, MainDialogProc);

    if (hMutex) {
        ReleaseMutex(hMutex);
        CloseHandle(hMutex);
    }
    return 0;
}

void UpdateTrayTip(HWND hWnd, const std::wstring& tipText) {
    if (!IsWindowVisible(hWnd)) { // Only update if the window is hidden (in tray)
        NOTIFYICONDATA nid = {};
        nid.cbSize = sizeof(NOTIFYICONDATA);
        nid.hWnd = hWnd;
        nid.uID = IDI_WECHATRECORDER;
        nid.uFlags = NIF_TIP;
        wcscpy_s(nid.szTip, tipText.c_str());
        Shell_NotifyIcon(NIM_MODIFY, &nid);
    }
}

void AddTrayIcon(HWND hWnd) {
    NOTIFYICONDATA nid = {};
    nid.cbSize = sizeof(NOTIFYICONDATA);
    nid.hWnd = hWnd;
    nid.uID = IDI_WECHATRECORDER;
    nid.uFlags = NIF_ICON | NIF_MESSAGE | NIF_TIP;
    nid.uCallbackMessage = WM_APP_TRAY_MSG;
    nid.hIcon = (HICON)LoadImage(hInst, MAKEINTRESOURCE(IDI_WECHATRECORDER), IMAGE_ICON,
        GetSystemMetrics(SM_CXSMICON), GetSystemMetrics(SM_CYSMICON), LR_DEFAULTCOLOR);

    wchar_t statusText[100];
    GetDlgItemText(hWnd, IDC_LABEL_STATUS, statusText, 100);
    wcscpy_s(nid.szTip, statusText);

    Shell_NotifyIcon(NIM_ADD, &nid);
}

void RemoveTrayIcon(HWND hWnd) {
    NOTIFYICONDATA nid = {};
    nid.cbSize = sizeof(NOTIFYICONDATA);
    nid.hWnd = hWnd;
    nid.uID = IDI_WECHATRECORDER;
    Shell_NotifyIcon(NIM_DELETE, &nid);
}

void ShowTrayMenu(HWND hWnd) {
    POINT pt;
    GetCursorPos(&pt);
    HMENU hMenu = LoadMenu(hInst, MAKEINTRESOURCE(IDR_TRAY_MENU));
    if (hMenu) {
        HMENU hSubMenu = GetSubMenu(hMenu, 0);
        if (hSubMenu) {
            SetForegroundWindow(hWnd);
            TrackPopupMenu(hSubMenu, TPM_LEFTALIGN | TPM_BOTTOMALIGN | TPM_RIGHTBUTTON, pt.x, pt.y, 0, hWnd, NULL);
        }
        DestroyMenu(hMenu);
    }
}

// 更新录音信息显示
void UpdateRecordingInfo(HWND hWnd) {
    if (!isRecording) return;
    
    // 获取录音时长
    int duration = g_recorder.GetRecordingDuration();
    int hours = duration / 3600;
    int minutes = (duration % 3600) / 60;
    int seconds = duration % 60;
    
    wchar_t durationText[64];
    swprintf_s(durationText, L"录音时长: %02d:%02d:%02d", hours, minutes, seconds);
    SetDlgItemText(hWnd, IDC_LABEL_DURATION, durationText);
    
    // 获取文件大小
    DWORD fileSize = g_recorder.GetCurrentFileSize();
    wchar_t fileSizeText[64];
    
    if (fileSize < 1024) {
        swprintf_s(fileSizeText, L"文件大小: %u B", fileSize);
    } else if (fileSize < 1024 * 1024) {
        swprintf_s(fileSizeText, L"文件大小: %.2f KB", fileSize / 1024.0);
    } else {
        swprintf_s(fileSizeText, L"文件大小: %.2f MB", fileSize / (1024.0 * 1024.0));
    }
    SetDlgItemText(hWnd, IDC_LABEL_FILESIZE, fileSizeText);
}

// 打开录音文件所在文件夹
void OpenRecordingFolder() {
    if (g_savePath.empty()) {
        MessageBoxW(NULL, L"录音保存路径未设置！", L"提示", MB_OK | MB_ICONWARNING);
        return;
    }
    
    // 打开保存路径文件夹
    ShellExecuteW(NULL, L"open", g_savePath.c_str(), NULL, NULL, SW_SHOW);
}

// 开始手动录音
void StartManualRecording(HWND hWnd) {
    if (isRecording) {
        MessageBoxW(hWnd, L"已有录音正在进行中！", L"提示", MB_OK | MB_ICONWARNING);
        return;
    }
    
    std::wstring inputDeviceId = GetSelectedInputDeviceId();
    std::wstring outputDeviceId = GetSelectedOutputDeviceId();
    
    if (inputDeviceId.empty() || outputDeviceId.empty()) {
        // ======================================
        // 【唯一修改行】：双引号转义 \"设置\"
        // ======================================
        MessageBoxW(hWnd, L"请在\"设置\"中选择有效的输入和输出设备。", L"错误", MB_OK | MB_ICONERROR);
        return;
    }
    
    // 生成手动录音文件路径
    std::wstring filePath = GenerateSavePath(g_savePath, L"手动录音");
    std::filesystem::create_directories(std::filesystem::path(filePath).parent_path());
    
    if (g_recorder.Start(filePath, inputDeviceId, outputDeviceId, g_micVolumePercent, g_speakerVolumePercent)) {
        isRecording = true;
        isManualRecording = true;
        g_currentRecordingPath = filePath;
        
        SetDlgItemText(hWnd, IDC_LABEL_STATUS, L"手动录音中...");
        SetDlgItemText(hWnd, IDC_BTN_MANUAL_RECORD, L"停止录音");
        UpdateTrayTip(hWnd, L"手动录音中...");
        
        // 启动UI更新定时器
        SetTimer(hWnd, TIMER_UPDATE_UI, 500, NULL);
        
        WriteLog(L"[主线程] 手动录音已开始");
    } else {
        MessageBoxW(hWnd, L"启动录音失败！", L"错误", MB_OK | MB_ICONERROR);
    }
}

// 停止手动录音
void StopManualRecording(HWND hWnd) {
    if (!isManualRecording) return;
    
    StopRecording(hWnd);
    SetDlgItemText(hWnd, IDC_BTN_MANUAL_RECORD, L"手动录音");
    
    WriteLog(L"[主线程] 手动录音已停止");
    MessageBoxW(hWnd, (L"录音已保存至：\n" + g_currentRecordingPath).c_str(), L"录音完成", MB_OK | MB_ICONINFORMATION);
}

