/*
 * =====================================================================================
 *
 * Filename:  wasapi_recorder.cpp
 *
 * Description:  WASAPI-based audio recording implementation
 *
 * =====================================================================================
 */

#include "wasapi_recorder.h"
#include "log.h"
#include <filesystem>
#include <sstream>
#include <shlwapi.h>
#include <mfapi.h>
#include <mfidl.h>
#include <mfreadwrite.h>
#include <propvarutil.h>

#pragma comment(lib, "Shlwapi.lib")
#pragma comment(lib, "Mfplat.lib")
#pragma comment(lib, "Mfreadwrite.lib")
#pragma comment(lib, "Mfuuid.lib")

// Constructor
WasapiRecorder::WasapiRecorder()
    : m_isRecording(false)
    , m_micVolumePercent(100)
    , m_speakerVolumePercent(100)
{
}

// Destructor
WasapiRecorder::~WasapiRecorder() {
    Stop();
}

// Start recording
bool WasapiRecorder::Start(
    const std::wstring& filename,
    const std::wstring& inputDeviceId,
    const std::wstring& outputDeviceId,
    int micVolumePercent,
    int speakerVolumePercent)
{
    if (m_isRecording.load()) {
        WriteLog(L"[WasapiRecorder] Already recording, cannot start again");
        return false;
    }

    WriteLog(L"[WasapiRecorder] Starting recording...");
    WriteLog(L"  Output file: %s", filename.c_str());
    WriteLog(L"  Mic volume: %d%%", micVolumePercent);
    WriteLog(L"  Speaker volume: %d%%", speakerVolumePercent);

    m_finalMp3Path = filename;
    m_inputDeviceId = inputDeviceId;
    m_outputDeviceId = outputDeviceId;
    m_micVolumePercent = micVolumePercent;
    m_speakerVolumePercent = speakerVolumePercent;

    // Generate temporary WAV file paths
    std::filesystem::path finalPath(filename);
    std::filesystem::path parentDir = finalPath.parent_path();
    std::wstring baseName = finalPath.stem().wstring();

    m_micTempWavPath = (parentDir / (baseName + L"_mic.wav")).wstring();
    m_speakerTempWavPath = (parentDir / (baseName + L"_speaker.wav")).wstring();

    WriteLog(L"  Temp mic WAV: %s", m_micTempWavPath.c_str());
    WriteLog(L"  Temp speaker WAV: %s", m_speakerTempWavPath.c_str());

    // Start recording flag
    m_isRecording.store(true);

    // Start recording threads
    try {
        m_micThread = std::thread(&WasapiRecorder::MicRecordThreadProc, this);
        m_speakerThread = std::thread(&WasapiRecorder::SpeakerRecordThreadProc, this);
        
        WriteLog(L"[WasapiRecorder] Recording threads started successfully");
        return true;
    }
    catch (const std::exception& e) {
        WriteLog(L"[WasapiRecorder] Failed to start recording threads: %S", e.what());
        m_isRecording.store(false);
        return false;
    }
}

// Stop recording
void WasapiRecorder::Stop() {
    if (!m_isRecording.load()) {
        return;
    }

    WriteLog(L"[WasapiRecorder] Stopping recording...");
    m_isRecording.store(false);

    // Wait for threads to finish
    if (m_micThread.joinable()) {
        m_micThread.join();
    }
    if (m_speakerThread.joinable()) {
        m_speakerThread.join();
    }

    WriteLog(L"[WasapiRecorder] Recording stopped, merging audio files...");

    // Merge the two WAV files and convert to MP3 using FFmpeg
    std::wstringstream cmdLine;
    cmdLine << L"ffmpeg -y "
            << L"-i \"" << m_micTempWavPath << L"\" "
            << L"-i \"" << m_speakerTempWavPath << L"\" "
            << L"-filter_complex \"[0:a]volume=" << (m_micVolumePercent / 100.0) << L"[mic];"
            << L"[1:a]volume=" << (m_speakerVolumePercent / 100.0) << L"[spk];"
            << L"[mic][spk]amix=inputs=2:duration=longest\" "
            << L"-b:a 192k "
            << L"\"" << m_finalMp3Path << L"\"";

    RunFFmpegAndLog(cmdLine.str());

    // Delete temporary WAV files
    try {
        if (std::filesystem::exists(m_micTempWavPath)) {
            std::filesystem::remove(m_micTempWavPath);
            WriteLog(L"[WasapiRecorder] Deleted temp file: %s", m_micTempWavPath.c_str());
        }
        if (std::filesystem::exists(m_speakerTempWavPath)) {
            std::filesystem::remove(m_speakerTempWavPath);
            WriteLog(L"[WasapiRecorder] Deleted temp file: %s", m_speakerTempWavPath.c_str());
        }
    }
    catch (const std::filesystem::filesystem_error& e) {
        WriteLog(L"[WasapiRecorder] Error deleting temp files: %S", e.what());
    }

    WriteLog(L"[WasapiRecorder] Recording complete: %s", m_finalMp3Path.c_str());
}

// Check if currently recording
bool WasapiRecorder::IsRecording() const {
    return m_isRecording.load();
}

// Microphone recording thread
void WasapiRecorder::MicRecordThreadProc() {
    WriteLog(L"[WasapiRecorder] Mic thread started");
    RecordLoop(true);
    WriteLog(L"[WasapiRecorder] Mic thread finished");
}

// Speaker recording thread
void WasapiRecorder::SpeakerRecordThreadProc() {
    WriteLog(L"[WasapiRecorder] Speaker thread started");
    RecordLoop(false);
    WriteLog(L"[WasapiRecorder] Speaker thread finished");
}

// Main recording loop (simplified implementation)
void WasapiRecorder::RecordLoop(bool isMic) {
    const std::wstring& deviceId = isMic ? m_inputDeviceId : m_outputDeviceId;
    const std::wstring& outputPath = isMic ? m_micTempWavPath : m_speakerTempWavPath;

    WriteLog(L"[WasapiRecorder] RecordLoop %s: device=%s, output=%s",
        isMic ? L"MIC" : L"SPEAKER", deviceId.c_str(), outputPath.c_str());

    // For now, use FFmpeg to capture audio
    std::wstringstream cmdLine;
    
    if (isMic) {
        // Record from microphone
        cmdLine << L"ffmpeg -f dshow -i audio=\"" << deviceId << L"\" "
                << L"-t 3600 "  // Max 1 hour
                << L"-y \"" << outputPath << L"\"";
    } else {
        // Record from speakers (loopback)
        cmdLine << L"ffmpeg -f dshow -i audio=\"" << deviceId << L"\" "
                << L"-t 3600 "  // Max 1 hour
                << L"-y \"" << outputPath << L"\"";
    }

    // Start FFmpeg process
    STARTUPINFOW si = { sizeof(si) };
    PROCESS_INFORMATION pi = { 0 };
    si.dwFlags = STARTF_USESHOWWINDOW;
    si.wShowWindow = SW_HIDE;

    std::wstring cmdLineCopy = cmdLine.str();
    
    if (CreateProcessW(NULL, &cmdLineCopy[0], NULL, NULL, FALSE,
                      CREATE_NO_WINDOW, NULL, NULL, &si, &pi)) {
        
        WriteLog(L"[WasapiRecorder] FFmpeg process started for %s",
            isMic ? L"MIC" : L"SPEAKER");

        // Wait for recording to stop or process to end
        while (m_isRecording.load()) {
            DWORD result = WaitForSingleObject(pi.hProcess, 100);
            if (result == WAIT_OBJECT_0) {
                // Process ended
                break;
            }
        }

        // Stop the process if still running
        if (m_isRecording.load() == false) {
            TerminateProcess(pi.hProcess, 0);
            WriteLog(L"[WasapiRecorder] Terminated FFmpeg process for %s",
                isMic ? L"MIC" : L"SPEAKER");
        }

        CloseHandle(pi.hProcess);
        CloseHandle(pi.hThread);
    } else {
        WriteLog(L"[WasapiRecorder] Failed to start FFmpeg process for %s: error %d",
            isMic ? L"MIC" : L"SPEAKER", GetLastError());
    }
}

// Run FFmpeg and log output
void WasapiRecorder::RunFFmpegAndLog(const std::wstring& cmdLine) {
    WriteLog(L"[WasapiRecorder] Running FFmpeg: %s", cmdLine.c_str());

    STARTUPINFOW si = { sizeof(si) };
    PROCESS_INFORMATION pi = { 0 };
    si.dwFlags = STARTF_USESHOWWINDOW;
    si.wShowWindow = SW_HIDE;

    std::wstring cmdLineCopy = cmdLine;
    
    if (CreateProcessW(NULL, &cmdLineCopy[0], NULL, NULL, FALSE,
                      CREATE_NO_WINDOW, NULL, NULL, &si, &pi)) {
        
        // Wait for process to complete
        WaitForSingleObject(pi.hProcess, INFINITE);
        
        DWORD exitCode = 0;
        GetExitCodeProcess(pi.hProcess, &exitCode);
        
        WriteLog(L"[WasapiRecorder] FFmpeg finished with exit code: %d", exitCode);
        
        CloseHandle(pi.hProcess);
        CloseHandle(pi.hThread);
    } else {
        WriteLog(L"[WasapiRecorder] Failed to run FFmpeg: error %d", GetLastError());
    }
}