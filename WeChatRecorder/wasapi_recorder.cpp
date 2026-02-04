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
HANDLE hUpdateInfoThread = NULL;
std::atomic<bool> monitorRunning = false;
std::atomic<bool> updateInfoRunning = false;
bool isRecording = false;
bool isMonitorStarted = false;
std::wstring g_lastMicAppName;
std::vector<std::wstring> g_blacklist;
std::wstring g_currentRecordingFile;
DWORD g_recordingStartTime = 0;

// --- Forward Declarations ---
void AddTrayIcon(HWND hWnd);
void RemoveTrayIcon(HWND hWnd);
void ShowTrayMenu(HWND hWnd);
void UpdateTrayTip(HWND hWnd, const std::wstring& tipText);
void UpdateRecordingInfo(HWND hWnd);
DWORD WINAPI UpdateInfoThreadProc(LPVOID lpParam);
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
        g_currentRecordingFile = filePath;
        g_recordingStartTime = GetTickCount();
        
        // 启动信息更新线程
        updateInfoRunning = true;
        hUpdateInfoThread = CreateThread(NULL, 0, UpdateInfoThreadProc, hWnd, 0, NULL);
        
        std::wstring statusText = L"正在录制: " + appName;
        SetDlgItemText(hWnd, IDC_LABEL_STATUS, statusText.c_str());
        UpdateTrayTip(hWnd, statusText);
    }
    else {
        isRecording = false;
        SetDlgItemText(hWnd, IDC_LABEL_STATUS, L"启动失败");
        UpdateTrayTip(hWnd, L"启动失败");
    }
}

void StopRecording(HWND hWnd) {
    if (!isRecording) return;
    
    // 停止信息更新线程
    updateInfoRunning = false;
    if (hUpdateInfoThread) {
        WaitForSingleObject(hUpdateInfoThread, 1000);
        CloseHandle(hUpdateInfoThread);
        hUpdateInfoThread = NULL;
    }
    
    g_recorder.Stop();
    isRecording = false;
    g_currentRecordingFile.clear();
    
    // 重置显示
    SetDlgItemText(hWnd, IDC_LABEL_DURATION, L"00:00:00");
    SetDlgItemText(hWnd, IDC_LABEL_FILESIZE, L"0 KB");

    std::wstring statusText = isMonitorStarted ? L"已启动检测..." : L"等待中...";
    SetDlgItemText(hWnd, IDC_LABEL_STATUS, statusText.c_str());
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

// 更新录音信息的线程
DWORD WINAPI UpdateInfoThreadProc(LPVOID lpParam) {
    HWND hWnd = (HWND)lpParam;
    while (updateInfoRunning && isRecording) {
        UpdateRecordingInfo(hWnd);
        Sleep(500);  // 每500ms更新一次
    }
    return 0;
}

// 更新录音时长和文件大小
void UpdateRecordingInfo(HWND hWnd) {
    if (!isRecording || g_currentRecordingFile.empty()) return;
    
    // 计算录音时长
    DWORD elapsedTime = (GetTickCount() - g_recordingStartTime) / 1000;  // 秒
    DWORD hours = elapsedTime / 3600;
    DWORD minutes = (elapsedTime % 3600) / 60;
    DWORD seconds = elapsedTime % 60;
    
    wchar_t timeBuffer[32];
    swprintf_s(timeBuffer, L"%02lu:%02lu:%02lu", hours, minutes, seconds);
    SetDlgItemText(hWnd, IDC_LABEL_DURATION, timeBuffer);
    
    // 计算文件大小
    std::wstring mp3File = g_currentRecordingFile;
    // 检查多个可能的临时文件
    std::vector<std::wstring> possibleFiles = {
        mp3File,
        mp3File + L".mic.wav",
        mp3File + L".speaker.wav"
    };
    
    unsigned long long totalSize = 0;
    for (const auto& file : possibleFiles) {
        try {
            if (std::filesystem::exists(file)) {
                totalSize += std::filesystem::file_size(file);
            }
        }
        catch (...) {
            // 忽略文件访问错误
        }
    }
    
    wchar_t sizeBuffer[64];
    if (totalSize < 1024) {
        swprintf_s(sizeBuffer, L"%llu B", totalSize);
    }
    else if (totalSize < 1024 * 1024) {
        swprintf_s(sizeBuffer, L"%.2f KB", totalSize / 1024.0);
    }
    else if (totalSize < 1024 * 1024 * 1024) {
        swprintf_s(sizeBuffer, L"%.2f MB", totalSize / (1024.0 * 1024.0));
    }
    else {
        swprintf_s(sizeBuffer, L"%.2f GB", totalSize / (1024.0 * 1024.0 * 1024.0));
    }
    
    SetDlgItemText(hWnd, IDC_LABEL_FILESIZE, sizeBuffer);
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
        SetDlgItemText(hDlg, IDC_LABEL_DURATION, L"00:00:00");
        SetDlgItemText(hDlg, IDC_LABEL_FILESIZE, L"0 KB");
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
    case WM_CLOSE:
        monitorRunning = false;
        updateInfoRunning = false;
        if (hMonitorThread) { WaitForSingleObject(hMonitorThread, 1000); CloseHandle(hMonitorThread); }
        if (hUpdateInfoThread) { WaitForSingleObject(hUpdateInfoThread, 1000); CloseHandle(hUpdateInfoThread); }
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