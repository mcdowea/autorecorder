@echo off
REM Windows 编译测试脚本

echo ==========================================
echo 自动录音程序 - Windows 编译测试
echo ==========================================
echo.

REM 检查 Rust 是否安装
echo [1/5] 检查 Rust 工具链...
rustc --version >nul 2>&1
if errorlevel 1 (
    echo [错误] 未找到 Rust，请先安装 Rust
    echo 访问: https://rustup.rs/
    pause
    exit /b 1
)
echo [成功] Rust 已安装
rustc --version
echo.

REM 检查 Cargo 是否安装
echo [2/5] 检查 Cargo...
cargo --version >nul 2>&1
if errorlevel 1 (
    echo [错误] 未找到 Cargo
    pause
    exit /b 1
)
echo [成功] Cargo 已安装
cargo --version
echo.

REM 清理之前的编译
echo [3/5] 清理之前的编译...
if exist target (
    echo 删除 target 目录...
    rmdir /s /q target
)
echo [成功] 清理完成
echo.

REM 编译项目
echo [4/5] 开始编译项目（发布版本，无 GUI）...
echo 这可能需要几分钟时间，请耐心等待...
echo.

cargo build --release --no-default-features --target x86_64-pc-windows-msvc

if errorlevel 1 (
    echo.
    echo [错误] 编译失败！
    echo.
    echo 常见问题解决方案:
    echo 1. 确保已安装 Visual Studio Build Tools
    echo 2. 检查网络连接（需要下载依赖）
    echo 3. 查看上面的错误信息
    echo.
    pause
    exit /b 1
)

echo.
echo [成功] 编译完成！
echo.

REM 检查可执行文件
echo [5/5] 验证可执行文件...
if exist target\x86_64-pc-windows-msvc\release\auto-audio-recorder.exe (
    echo [成功] 可执行文件已生成
    echo.
    echo 文件位置: target\x86_64-pc-windows-msvc\release\auto-audio-recorder.exe
    
    REM 显示文件大小
    for %%A in (target\x86_64-pc-windows-msvc\release\auto-audio-recorder.exe) do (
        echo 文件大小: %%~zA 字节
    )
    
    echo.
    echo ==========================================
    echo 编译成功！（命令行版本）
    echo ==========================================
    echo.
    echo 运行程序:
    echo   .\target\x86_64-pc-windows-msvc\release\auto-audio-recorder.exe run
    echo.
    echo 或手动录音:
    echo   .\target\x86_64-pc-windows-msvc\release\auto-audio-recorder.exe start
    echo.
    echo 查看帮助:
    echo   .\target\x86_64-pc-windows-msvc\release\auto-audio-recorder.exe --help
    echo.
    echo 注意: 此版本不包含 GUI，如需 GUI 请使用:
    echo   cargo build --release --features gui
    echo.
) else (
    echo [错误] 未找到可执行文件
    pause
    exit /b 1
)

pause
