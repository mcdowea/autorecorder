@echo off
:: Auto Recorder 诊断工具
chcp 65001 >nul 2>&1
color 0E

echo.
echo ========================================
echo  Auto Recorder 诊断工具
echo ========================================
echo.

:: 检查文件是否存在
echo [检查 1/6] 检查可执行文件...
if exist "auto-recorder.exe" (
    echo   ✓ auto-recorder.exe 存在
) else if exist "auto-recorder-windows-x64.exe" (
    echo   ✓ auto-recorder-windows-x64.exe 存在
    echo   → 重命名为 auto-recorder.exe
    copy "auto-recorder-windows-x64.exe" "auto-recorder.exe" >nul
) else (
    echo   ✗ 未找到可执行文件
    echo.
    echo   请确保以下文件之一存在：
    echo     - auto-recorder.exe
    echo     - auto-recorder-windows-x64.exe
    echo.
    goto :error_exit
)

echo.
echo [检查 2/6] 测试程序启动...
auto-recorder.exe --version 2>test_output.txt
if errorlevel 1 (
    echo   ✗ 程序无法启动
    echo.
    echo   错误信息：
    type test_output.txt
    echo.
    goto :check_dependencies
) else (
    echo   ✓ 程序可以启动
)

echo.
echo [检查 3/6] 检查依赖项...
:check_dependencies

:: 检查 MSVC 运行时
echo   检查 Visual C++ 运行时...
reg query "HKLM\SOFTWARE\Microsoft\VisualStudio\14.0\VC\Runtimes\x64" >nul 2>&1
if errorlevel 1 (
    echo   ⚠ 未检测到 VC++ 运行时
    echo.
    echo   解决方案：
    echo   下载并安装: https://aka.ms/vs/17/release/vc_redist.x64.exe
    echo.
    set has_error=1
) else (
    echo   ✓ VC++ 运行时已安装
)

echo.
echo [检查 4/6] 检查杀毒软件...
echo   请检查是否被杀毒软件拦截
echo   如被拦截，请添加到白名单

echo.
echo [检查 5/6] 检查音频设备...
auto-recorder.exe list-devices >device_list.txt 2>&1
if errorlevel 1 (
    echo   ⚠ 无法列出音频设备
    echo.
    type device_list.txt
    echo.
) else (
    echo   ✓ 音频设备检测正常
    echo.
    echo   可用设备：
    type device_list.txt | findstr /C:"Microphone" /C:"Stereo" /C:"立体声"
    echo.
)

echo.
echo [检查 6/6] 检查配置文件...
if exist "config.json" (
    echo   ✓ config.json 存在
) else (
    echo   ℹ config.json 不存在（这是正常的）
    echo   → 运行 'auto-recorder.exe gen-config' 生成
)

echo.
echo ========================================
echo  诊断完成
echo ========================================
echo.

if defined has_error (
    echo ⚠ 发现一些问题，请查看上面的信息
) else (
    echo ✓ 所有检查通过！
    echo.
    echo 下一步：
    echo   1. 双击 launcher.bat 使用图形界面
    echo   2. 或在命令行运行: auto-recorder.exe --help
)

echo.

:: 清理临时文件
del test_output.txt 2>nul
del device_list.txt 2>nul

pause
exit /b 0

:error_exit
echo.
pause
exit /b 1
