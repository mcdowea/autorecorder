@echo off
chcp 65001 >nul 2>&1
setlocal enabledelayedexpansion

:: 设置颜色
color 0A

:menu
cls
echo.
echo ========================================
echo     Auto Recorder - 自动录音程序
echo ========================================
echo.
echo 请选择操作：
echo.
echo   [1] 生成配置文件
echo   [2] 查看音频设备列表
echo   [3] 手动录音
echo   [4] 自动录音（检测通话）
echo   [5] 转换 WAV 为 MP3
echo   [6] 测试程序是否正常
echo   [0] 退出
echo.
echo ========================================
echo.

set /p choice="请输入选项 (0-6): "

if "%choice%"=="0" goto :exit
if "%choice%"=="1" goto :config
if "%choice%"=="2" goto :devices
if "%choice%"=="3" goto :record
if "%choice%"=="4" goto :auto
if "%choice%"=="5" goto :convert
if "%choice%"=="6" goto :test

echo.
echo [错误] 无效的选项，请重新选择
timeout /t 2 >nul
goto :menu

:test
cls
echo.
echo ========================================
echo 测试程序
echo ========================================
echo.
echo 正在测试 auto-recorder.exe...
echo.

if not exist "auto-recorder.exe" (
    echo [错误] auto-recorder.exe 不存在！
    echo.
    echo 请确保以下文件存在：
    echo   - auto-recorder.exe
    echo   - auto-recorder-windows-x64.exe
    echo.
    goto :test_end
)

echo [1/3] 检查程序是否可执行...
auto-recorder.exe --version 2>nul
if errorlevel 1 (
    echo [失败] 程序无法运行
    echo.
    echo 可能原因：
    echo   1. 缺少 Visual C++ 运行时
    echo   2. 被杀毒软件拦截
    echo   3. 文件损坏
    echo.
    echo 解决方案：
    echo   - 安装 VC++ Redistributable: https://aka.ms/vs/17/release/vc_redist.x64.exe
    echo   - 将此文件夹添加到杀毒软件白名单
    echo.
    goto :test_end
) else (
    echo [成功] 程序可以运行
)

echo.
echo [2/3] 检查帮助信息...
auto-recorder.exe --help >nul 2>&1
if errorlevel 1 (
    echo [警告] 帮助信息获取失败
) else (
    echo [成功] 帮助信息正常
)

echo.
echo [3/3] 检查配置文件...
if exist "config.json" (
    echo [成功] 配置文件存在
) else (
    echo [提示] 配置文件不存在，建议先生成配置
)

echo.
echo ========================================
echo 测试完成！程序运行正常。
echo ========================================

:test_end
echo.
pause
goto :menu

:config
cls
echo.
echo ========================================
echo 生成配置文件
echo ========================================
echo.

if exist "config.json" (
    echo [警告] config.json 已存在
    set /p overwrite="是否覆盖? (Y/N): "
    if /i not "!overwrite!"=="Y" (
        echo 已取消操作
        timeout /t 2 >nul
        goto :menu
    )
)

echo 正在生成配置文件...
auto-recorder.exe gen-config

if errorlevel 1 (
    echo.
    echo [错误] 配置文件生成失败
) else (
    echo.
    echo [成功] 配置文件已生成: config.json
    echo.
    echo 您可以编辑 config.json 来自定义设置
)

echo.
pause
goto :menu

:devices
cls
echo.
echo ========================================
echo 音频设备列表
echo ========================================
echo.
echo 正在扫描音频设备...
echo.

auto-recorder.exe list-devices

echo.
echo ========================================
echo.
echo [重要] Windows 用户需要启用"立体声混音"
echo.
echo 步骤：
echo   1. 右键点击音量图标 → 声音设置
echo   2. 更多声音设置 → 录制标签
echo   3. 右键空白处 → 显示已禁用的设备
echo   4. 找到"立体声混音" → 右键启用
echo.
pause
goto :menu

:record
cls
echo.
echo ========================================
echo 手动录音模式
echo ========================================
echo.
echo 录音设置：
echo   - 格式: WAV (无损)
echo   - 采样率: 44100 Hz
echo   - 保存位置: recordings\
echo.
echo [提示] 按 Ctrl+C 停止录音
echo.
echo 是否开始录音？
pause

echo.
echo 录音中... (按 Ctrl+C 停止)
echo.

auto-recorder.exe record

echo.
echo 录音已停止
echo.
echo 录音文件保存在: recordings\
dir /b recordings\*.wav 2>nul | findstr /r ".*" >nul && (
    echo.
    echo 最新的录音文件：
    dir /b /o-d recordings\*.wav 2>nul | findstr /r /n "^" | findstr /r "^1:"
) || (
    echo [提示] recordings 目录为空
)

echo.
pause
goto :menu

:auto
cls
echo.
echo ========================================
echo 自动录音模式
echo ========================================
echo.
echo 此模式将：
echo   1. 持续监控通话应用（微信、QQ、Teams等）
echo   2. 检测到通话时自动开始录音
echo   3. 通话结束后自动停止录音
echo.
echo [注意]
echo   - 仅支持 Windows 系统
echo   - 需要启用"立体声混音"设备
echo   - 程序将在后台运行
echo.
echo 按任意键开始自动监控...
pause >nul

echo.
echo 正在启动自动监控...
echo 程序将持续运行，检测通话应用
echo.
echo [按 Ctrl+C 停止监控]
echo.

auto-recorder.exe auto

echo.
pause
goto :menu

:convert
cls
echo.
echo ========================================
echo 转换 WAV 为 MP3
echo ========================================
echo.

where ffmpeg >nul 2>&1
if errorlevel 1 (
    echo [错误] 未找到 ffmpeg
    echo.
    echo 请先安装 ffmpeg:
    echo   1. 访问 https://ffmpeg.org/download.html
    echo   2. 下载 Windows 版本
    echo   3. 解压并添加到系统 PATH
    echo.
    echo 或者使用 Chocolatey 安装:
    echo   choco install ffmpeg
    echo.
    pause
    goto :menu
)

echo 已找到 ffmpeg
echo.
echo 转换选项：
echo   [1] 标准质量 (128 kbps)
echo   [2] 高质量 (192 kbps)
echo   [3] 最高质量 (320 kbps)
echo   [0] 返回
echo.

set /p quality="请选择质量 (0-3): "

if "%quality%"=="0" goto :menu
if "%quality%"=="1" set bitrate=128k
if "%quality%"=="2" set bitrate=192k
if "%quality%"=="3" set bitrate=320k

if not defined bitrate (
    echo 无效选项
    timeout /t 2 >nul
    goto :convert
)

echo.
echo 正在转换...
echo.

call convert_to_mp3.bat -b %bitrate%

echo.
pause
goto :menu

:exit
cls
echo.
echo 感谢使用 Auto Recorder！
echo.
timeout /t 2 >nul
exit /b 0
