#pragma once
#include <string>
#include <thread>
#include <atomic>
#include <windows.h>
#include <audioclient.h>
#include <mmdeviceapi.h>

class WasapiRecorder {
public:
    WasapiRecorder();
    ~WasapiRecorder();

    // --- �����޸��㣺Start �������ڽ��������ٷֱ� ---
    bool Start(const std::wstring& filename,
        const std::wstring& inputDeviceId,
        const std::wstring& outputDeviceId,
        int micVolumePercent,
        int speakerVolumePercent);
    void Stop();
    bool IsRecording() const;
    
    // 获取录音时长（秒）
    int GetRecordingDuration() const;
    // 获取当前文件大小（字节）
    DWORD GetCurrentFileSize() const;

private:
    void MicRecordThreadProc();
    void SpeakerRecordThreadProc();
    void RecordLoop(bool isMic);
    void RunFFmpegAndLog(const std::wstring& cmdLine);

    std::atomic<bool> m_isRecording;
    std::thread m_micThread;
    std::thread m_speakerThread;

    std::wstring m_finalMp3Path;
    std::wstring m_micTempWavPath;
    std::wstring m_speakerTempWavPath;
    std::wstring m_inputDeviceId;
    std::wstring m_outputDeviceId;
    int m_micVolumePercent;
    int m_speakerVolumePercent;
    
    // 录音时间追踪
    std::chrono::steady_clock::time_point m_startTime;
    std::atomic<DWORD> m_currentFileSize;
};