@echo off
echo ================================
echo Auto Recorder Build Script
echo ================================

where cargo >nul 2>nul
if %errorlevel% neq 0 (
    echo Error: Rust/Cargo not found. Please install from https://rustup.rs/
    exit /b 1
)

echo.
echo Step 1: Checking project structure...
if exist Cargo.toml (
    echo ✓ Cargo.toml found
) else (
    echo ✗ Cargo.toml not found
    exit /b 1
)

if exist src (
    echo ✓ src\ directory found
) else (
    echo ✗ src\ directory not found
    exit /b 1
)

echo.
echo Step 2: Checking dependencies...
cargo --version
rustc --version

echo.
echo Step 3: Running cargo check...
cargo check
if %errorlevel% neq 0 (
    echo ✗ Cargo check failed
    exit /b 1
)
echo ✓ Cargo check passed

echo.
echo Step 4: Building project (debug)...
cargo build
if %errorlevel% neq 0 (
    echo ✗ Debug build failed
    exit /b 1
)
echo ✓ Debug build successful

echo.
echo Step 5: Building project (release)...
cargo build --release
if %errorlevel% neq 0 (
    echo ✗ Release build failed
    exit /b 1
)
echo ✓ Release build successful

echo.
echo ================================
echo Build completed successfully!
echo ================================
echo.
echo Binary locations:
echo   Debug:   .\target\debug\auto-recorder.exe
echo   Release: .\target\release\auto-recorder.exe
echo.
echo Quick start:
echo   .\target\release\auto-recorder.exe gen-config
echo   .\target\release\auto-recorder.exe list-devices
echo   .\target\release\auto-recorder.exe record
echo.
pause
