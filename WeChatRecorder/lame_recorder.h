#ifndef LAME_RECORDER_H
#define LAME_RECORDER_H

#include <Windows.h>
#include <mmdeviceapi.h>
#include <audioclient.h>
#include <string>
#include <atomic>
#include <thread>
#include <cstdio>

// Forward declaration of lame_t
struct lame_global_struct;
typedef struct lame_global_struct* lame_t;

// LAME编码器录音类 - 替代FFmpeg方案
class LameRecorder {
public:
    LameRecorder();
    ~LameRecorder();

    // 开始录音
    bool Start(const std::wstring& outputPath,
        const std::wstring& inputDeviceId,
        const std::wstring& outputDeviceId,
        int micVolume,
        int speakerVolume);

    // 停止录音
    void Stop();

    // 是否正在录音
    bool IsRecording() const { return m_recording; }

    // 获取录音时长(秒)
    int GetRecordingDuration() const;

    // 获取当前文件大小(字节)
    DWORD GetCurrentFileSize() const;

private:
    // WASAPI相关
    struct AudioDevice {
        IMMDevice* pDevice;
        IAudioClient* pAudioClient;
        IAudioCaptureClient* pCaptureClient;
        WAVEFORMATEX* pWaveFormat;
        HANDLE hAudioSamplesReadyEvent;

        AudioDevice() : pDevice(nullptr), pAudioClient(nullptr),
            pCaptureClient(nullptr), pWaveFormat(nullptr),
            hAudioSamplesReadyEvent(nullptr) {
        }
    };

    AudioDevice m_inputDevice;   // 麦克风设备
    AudioDevice m_outputDevice;  // 扬声器设备

    // LAME编码器
    lame_t m_lame;

    // 文件写入
    FILE* m_outputFile = nullptr;
    std::wstring m_outputPath;

    // 线程控制
    std::atomic<bool> m_recording{ false };
    std::thread m_recordThread;
    std::thread m_encodeThread;

    // 音频缓冲区大小常量
    enum { BUFFER_SIZE = 8192 };
    BYTE m_mixedBuffer[BUFFER_SIZE * 4];  // 混音缓冲区

    // 统计信息
    std::atomic<int> m_duration{ 0 };
    std::atomic<DWORD> m_fileSize{ 0 };
    DWORD m_startTime = 0;

    // 音量控制
    int m_micVolume = 100;
    int m_speakerVolume = 100;

    // 私有方法
    bool InitializeAudioDevice(AudioDevice& device, const std::wstring& deviceId, bool isLoopback);
    void CleanupAudioDevice(AudioDevice& device);
    void RecordThreadProc();
    void MixAudioSamples(const float* inputSamples, int inputFrames,
        const float* outputSamples, int outputFrames,
        short* mixedSamples, int& mixedFrames);
    void ApplyVolume(float* samples, int frameCount, int volumePercent);
};

#endif // LAME_RECORDER_H