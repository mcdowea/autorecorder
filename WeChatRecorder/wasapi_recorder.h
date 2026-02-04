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

    // --- ºËĞÄĞŞ¸´µã£ºStart º¯ÊıÏÖÔÚ½ÓÊÕÒôÁ¿°Ù·Ö±È ---
    bool Start(const std::wstring& filename,
        const std::wstring& inputDeviceId,
        const std::wstring& outputDeviceId,
        int micVolumePercent,
        int speakerVolumePercent);
    void Stop();
    bool IsRecording() const;
    
    // è·å–å½•éŸ³æ—¶é•¿ï¼ˆç§’ï¼‰
    int GetRecordingDuration() const;
    // è·å–å½“å‰æ–‡ä»¶å¤§å°ï¼ˆå­—èŠ‚ï¼‰
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
    
    // å½•éŸ³æ—¶é—´è¿½è¸ª
    std::chrono::steady_clock::time_point m_startTime;
    std::atomic<DWORD> m_currentFileSize;
};