@echo off
:: 创建桌面快捷方式

echo.
echo ========================================
echo 创建桌面快捷方式
echo ========================================
echo.

set SCRIPT_DIR=%~dp0
set DESKTOP=%USERPROFILE%\Desktop

echo 当前目录: %SCRIPT_DIR%
echo 桌面路径: %DESKTOP%
echo.

:: 创建 VBS 脚本来创建快捷方式
echo Set oWS = WScript.CreateObject("WScript.Shell") > CreateShortcut.vbs
echo sLinkFile = "%DESKTOP%\Auto Recorder.lnk" >> CreateShortcut.vbs
echo Set oLink = oWS.CreateShortcut(sLinkFile) >> CreateShortcut.vbs
echo oLink.TargetPath = "%SCRIPT_DIR%launcher.bat" >> CreateShortcut.vbs
echo oLink.WorkingDirectory = "%SCRIPT_DIR%" >> CreateShortcut.vbs
echo oLink.Description = "Auto Recorder - 自动录音程序" >> CreateShortcut.vbs
echo oLink.IconLocation = "%SystemRoot%\System32\SoundRecorder.exe, 0" >> CreateShortcut.vbs
echo oLink.Save >> CreateShortcut.vbs

:: 运行 VBS 脚本
cscript //nologo CreateShortcut.vbs

:: 清理
del CreateShortcut.vbs

if exist "%DESKTOP%\Auto Recorder.lnk" (
    echo ✓ 快捷方式已创建！
    echo.
    echo 快捷方式位置: %DESKTOP%\Auto Recorder.lnk
    echo.
    echo 您现在可以：
    echo   1. 双击桌面上的 "Auto Recorder" 图标启动程序
    echo   2. 或继续使用 launcher.bat
    echo.
) else (
    echo ✗ 快捷方式创建失败
    echo.
    echo 请手动创建快捷方式：
    echo   1. 右键 launcher.bat → 发送到 → 桌面快捷方式
    echo.
)

pause
