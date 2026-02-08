@echo off
:: Auto Recorder GUI 启动脚本

echo.
echo ========================================
echo   Auto Recorder - 图形界面版本
echo ========================================
echo.

if exist "auto-recorder-gui.exe" (
    echo 启动图形界面...
    start "" "auto-recorder-gui.exe"
) else if exist "target\release\auto-recorder-gui.exe" (
    echo 启动图形界面...
    start "" "target\release\auto-recorder-gui.exe"
) else (
    echo [错误] 未找到 auto-recorder-gui.exe
    echo.
    echo 请先编译程序：
    echo   cargo build --release --bin auto-recorder-gui
    echo.
    pause
)
