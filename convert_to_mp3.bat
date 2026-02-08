@echo off
REM WAV to MP3 Converter Script for Windows
REM Requires ffmpeg to be installed and in PATH

where ffmpeg >nul 2>nul
if %errorlevel% neq 0 (
    echo Error: ffmpeg not found. Please install ffmpeg first.
    echo.
    echo Installation:
    echo   Download from https://ffmpeg.org/download.html
    echo   Add ffmpeg.exe to your system PATH
    pause
    exit /b 1
)

REM Default values
set INPUT_DIR=recordings
set OUTPUT_DIR=recordings\mp3
set BITRATE=128k
set QUALITY=2

REM Parse arguments
:parse_args
if "%~1"=="" goto end_parse
if "%~1"=="-i" set INPUT_DIR=%~2& shift & shift & goto parse_args
if "%~1"=="--input" set INPUT_DIR=%~2& shift & shift & goto parse_args
if "%~1"=="-o" set OUTPUT_DIR=%~2& shift & shift & goto parse_args
if "%~1"=="--output" set OUTPUT_DIR=%~2& shift & shift & goto parse_args
if "%~1"=="-b" set BITRATE=%~2& shift & shift & goto parse_args
if "%~1"=="--bitrate" set BITRATE=%~2& shift & shift & goto parse_args
if "%~1"=="-q" set QUALITY=%~2& shift & shift & goto parse_args
if "%~1"=="--quality" set QUALITY=%~2& shift & shift & goto parse_args
if "%~1"=="-h" goto show_help
if "%~1"=="--help" goto show_help
echo Unknown option: %~1
goto show_help

:show_help
echo Usage: %~nx0 [OPTIONS]
echo.
echo Options:
echo   -i, --input DIR     Input directory (default: recordings)
echo   -o, --output DIR    Output directory (default: recordings\mp3)
echo   -b, --bitrate RATE  MP3 bitrate (default: 128k)
echo   -q, --quality NUM   MP3 quality 0-9 (default: 2)
echo   -h, --help          Show this help message
echo.
echo Examples:
echo   %~nx0                                    # Convert all WAV files
echo   %~nx0 -b 192k -q 1                      # High quality conversion
echo   %~nx0 -i .\my-recordings -o .\output    # Custom directories
pause
exit /b 0

:end_parse

REM Create output directory
if not exist "%OUTPUT_DIR%" mkdir "%OUTPUT_DIR%"

REM Convert all WAV files
set count=0
for %%f in ("%INPUT_DIR%\*.wav") do (
    set "wav_file=%%f"
    set "filename=%%~nf"
    set "mp3_file=%OUTPUT_DIR%\%%~nf.mp3"
    
    echo Converting: %%f
    echo         to: !mp3_file!
    
    ffmpeg -i "%%f" -vn -ar 44100 -ac 1 -b:a %BITRATE% -q:a %QUALITY% "!mp3_file!" -y 2>nul
    
    if !errorlevel! equ 0 (
        echo [OK] Converted successfully
        set /a count+=1
    ) else (
        echo [FAIL] Conversion failed
    )
    echo.
)

if %count% equ 0 (
    echo No WAV files found in %INPUT_DIR%
) else (
    echo ======================================
    echo Conversion complete!
    echo Converted %count% file(s^)
    echo Output directory: %OUTPUT_DIR%
    echo ======================================
)

pause
