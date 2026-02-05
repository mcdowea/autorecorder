#include "lame_recorder.h"
#include "log.h"
#include "libs/lame/lame.h"
#include <comdef.h>
#include <propvarutil.h>
#include <functiondiscoverykeys_devpkey.h>
#include <algorithm>

#pragma comment(lib, "libs/lame/libmp3lame.lib")

LameRecorder::LameRecorder() : m_lame(nullptr), m_outputFile(nullptr),
m_startTime(0) {
}

LameRecorder::~LameRecorder() {
    Stop();
}

bool LameRecorder::Start(const std::wstring& outputPath,
    const std::wstring& inputDeviceId,
    const std::wstring& outputDeviceId,
    int micVolume,
    int speakerVolume) {
    if (m_recording) {
        WriteLog(L"[LameRecorder] 录音已在进行中");
        return false;
    }

    m_outputPath = outputPath;
    m_micVolume = micVolume;
    m_speakerVolume = speakerVolume;

    // 初始化COM
    CoInitialize(NULL);

    // 初始化音频设备
    if (!InitializeAudioDevice(m_inputDevice, inputDeviceId, false)) {
        WriteLog(L"[LameRecorder] 初始化输入设备失败");
        return false;
    }

    if (!InitializeAudioDevice(m_outputDevice, outputDeviceId, true)) {
        WriteLog(L"[LameRecorder] 初始化输出设备失败");
        CleanupAudioDevice(m_inputDevice);
        return false;
    }

    // 初始化LAME编码器
    m_lame = lame_init();
    if (!m_lame) {
        WriteLog(L"[LameRecorder] LAME初始化失败");
        CleanupAudioDevice(m_inputDevice);
        CleanupAudioDevice(m_outputDevice);
        return false;
    }

    // 配置LAME参数
    lame_set_in_samplerate(m_lame, 48000);  // 输入采样率
    lame_set_num_channels(m_lame, 2);       // 立体声
    // lame_set_out_samplerate(m_lame, 48000); // 输出采样率
    lame_set_brate(m_lame, 128);            // 比特率 128kbps
    // lame_set_mode(m_lame, STEREO);
    // lame_set_quality(m_lame, 2);            // 质量(0=最好, 9=最差)

    if (lame_init_params(m_lame) < 0) {
        WriteLog(L"[LameRecorder] LAME参数初始化失败");
        lame_close(m_lame);
        m_lame = nullptr;
        CleanupAudioDevice(m_inputDevice);
        CleanupAudioDevice(m_outputDevice);
        return false;
    }

    // 打开输出文件
    _wfopen_s(&m_outputFile, outputPath.c_str(), L"wb");
    if (!m_outputFile) {
        WriteLog(L"[LameRecorder] 无法创建输出文件: %s", outputPath.c_str());
        lame_close(m_lame);
        m_lame = nullptr;
        CleanupAudioDevice(m_inputDevice);
        CleanupAudioDevice(m_outputDevice);
        return false;
    }

    //     // 写入MP3文件头
    //     unsigned char mp3Header[7200];
    //     int headerSize = lame_get_id3v2_tag(m_lame, mp3Header, sizeof(mp3Header));
    //     if (headerSize > 0) {
    //         fwrite(mp3Header, 1, headerSize, m_outputFile);
    //         m_fileSize += headerSize;
    //     }

    // 启动音频设备
    HRESULT hr;
    hr = m_inputDevice.pAudioClient->Start();
    if (FAILED(hr)) {
        WriteLog(L"[LameRecorder] 启动输入设备失败: 0x%08X", hr);
        fclose(m_outputFile);
        m_outputFile = nullptr;
        lame_close(m_lame);
        m_lame = nullptr;
        CleanupAudioDevice(m_inputDevice);
        CleanupAudioDevice(m_outputDevice);
        return false;
    }

    hr = m_outputDevice.pAudioClient->Start();
    if (FAILED(hr)) {
        WriteLog(L"[LameRecorder] 启动输出设备失败: 0x%08X", hr);
        m_inputDevice.pAudioClient->Stop();
        fclose(m_outputFile);
        m_outputFile = nullptr;
        lame_close(m_lame);
        m_lame = nullptr;
        CleanupAudioDevice(m_inputDevice);
        CleanupAudioDevice(m_outputDevice);
        return false;
    }

    // 启动录音线程
    m_recording = true;
    m_startTime = GetTickCount();
    m_duration = 0;
    m_recordThread = std::thread(&LameRecorder::RecordThreadProc, this);

    WriteLog(L"[LameRecorder] 开始录音: %s", outputPath.c_str());
    return true;
}

void LameRecorder::Stop() {
    if (!m_recording) return;

    WriteLog(L"[LameRecorder] 停止录音");
    m_recording = false;

    // 等待录音线程结束
    if (m_recordThread.joinable()) {
        m_recordThread.join();
    }

    // 停止音频设备
    if (m_inputDevice.pAudioClient) {
        m_inputDevice.pAudioClient->Stop();
    }
    if (m_outputDevice.pAudioClient) {
        m_outputDevice.pAudioClient->Stop();
    }

    // 写入MP3尾部
    if (m_lame && m_outputFile) {
        unsigned char mp3Buffer[7200];
        int flushBytes = lame_encode_flush(m_lame, mp3Buffer, sizeof(mp3Buffer));
        if (flushBytes > 0) {
            fwrite(mp3Buffer, 1, flushBytes, m_outputFile);
            m_fileSize += flushBytes;
        }

        //         // 写入ID3v1标签
        //         int tagSize = lame_get_id3v1_tag(m_lame, mp3Buffer, sizeof(mp3Buffer));
        //         if (tagSize > 0) {
        //             fwrite(mp3Buffer, 1, tagSize, m_outputFile);
        //             m_fileSize += tagSize;
        //         }
    }

    // 关闭文件
    if (m_outputFile) {
        fclose(m_outputFile);
        m_outputFile = nullptr;
    }

    // 清理LAME
    if (m_lame) {
        lame_close(m_lame);
        m_lame = nullptr;
    }

    // 清理音频设备
    CleanupAudioDevice(m_inputDevice);
    CleanupAudioDevice(m_outputDevice);

    CoUninitialize();
}

int LameRecorder::GetRecordingDuration() const {
    if (!m_recording) return m_duration;
    return (GetTickCount() - m_startTime) / 1000;
}

DWORD LameRecorder::GetCurrentFileSize() const {
    return m_fileSize;
}

bool LameRecorder::InitializeAudioDevice(AudioDevice& device, const std::wstring& deviceId, bool isLoopback) {
    HRESULT hr;

    // 获取设备枚举器
    IMMDeviceEnumerator* pEnumerator = nullptr;
    hr = CoCreateInstance(__uuidof(MMDeviceEnumerator), NULL, CLSCTX_ALL,
        __uuidof(IMMDeviceEnumerator), (void**)&pEnumerator);
    if (FAILED(hr)) {
        WriteLog(L"[LameRecorder] 创建设备枚举器失败: 0x%08X", hr);
        return false;
    }

    // 获取指定设备
    hr = pEnumerator->GetDevice(deviceId.c_str(), &device.pDevice);
    pEnumerator->Release();

    if (FAILED(hr)) {
        WriteLog(L"[LameRecorder] 获取设备失败: %s, 0x%08X", deviceId.c_str(), hr);
        return false;
    }

    // 激活音频客户端
    hr = device.pDevice->Activate(__uuidof(IAudioClient), CLSCTX_ALL, NULL,
        (void**)&device.pAudioClient);
    if (FAILED(hr)) {
        WriteLog(L"[LameRecorder] 激活音频客户端失败: 0x%08X", hr);
        device.pDevice->Release();
        device.pDevice = nullptr;
        return false;
    }

    // 获取音频格式
    hr = device.pAudioClient->GetMixFormat(&device.pWaveFormat);
    if (FAILED(hr)) {
        WriteLog(L"[LameRecorder] 获取音频格式失败: 0x%08X", hr);
        device.pAudioClient->Release();
        device.pAudioClient = nullptr;
        device.pDevice->Release();
        device.pDevice = nullptr;
        return false;
    }

    // 创建事件
    device.hAudioSamplesReadyEvent = CreateEvent(NULL, FALSE, FALSE, NULL);
    if (!device.hAudioSamplesReadyEvent) {
        WriteLog(L"[LameRecorder] 创建事件失败");
        CoTaskMemFree(device.pWaveFormat);
        device.pAudioClient->Release();
        device.pDevice->Release();
        return false;
    }

    // 初始化音频客户端
    REFERENCE_TIME bufferDuration = 10000000; // 1秒
    DWORD streamFlags = AUDCLNT_STREAMFLAGS_EVENTCALLBACK;
    if (isLoopback) {
        streamFlags |= AUDCLNT_STREAMFLAGS_LOOPBACK;
    }
    hr = device.pAudioClient->Initialize(
        AUDCLNT_SHAREMODE_SHARED,
        streamFlags,
        bufferDuration,
        0,
        device.pWaveFormat,
        NULL);

    if (FAILED(hr)) {
        WriteLog(L"[LameRecorder] 初始化音频客户端失败: 0x%08X", hr);
        CloseHandle(device.hAudioSamplesReadyEvent);
        CoTaskMemFree(device.pWaveFormat);
        device.pAudioClient->Release();
        device.pDevice->Release();
        return false;
    }

    // 设置事件回调
    hr = device.pAudioClient->SetEventHandle(device.hAudioSamplesReadyEvent);
    if (FAILED(hr)) {
        WriteLog(L"[LameRecorder] 设置事件句柄失败: 0x%08X", hr);
        CloseHandle(device.hAudioSamplesReadyEvent);
        CoTaskMemFree(device.pWaveFormat);
        device.pAudioClient->Release();
        device.pDevice->Release();
        return false;
    }

    // 获取捕获客户端
    hr = device.pAudioClient->GetService(__uuidof(IAudioCaptureClient),
        (void**)&device.pCaptureClient);
    if (FAILED(hr)) {
        WriteLog(L"[LameRecorder] 获取捕获客户端失败: 0x%08X", hr);
        CloseHandle(device.hAudioSamplesReadyEvent);
        CoTaskMemFree(device.pWaveFormat);
        device.pAudioClient->Release();
        device.pDevice->Release();
        return false;
    }

    return true;
}

void LameRecorder::CleanupAudioDevice(AudioDevice& device) {
    if (device.pCaptureClient) {
        device.pCaptureClient->Release();
        device.pCaptureClient = nullptr;
    }
    if (device.hAudioSamplesReadyEvent) {
        CloseHandle(device.hAudioSamplesReadyEvent);
        device.hAudioSamplesReadyEvent = nullptr;
    }
    if (device.pWaveFormat) {
        CoTaskMemFree(device.pWaveFormat);
        device.pWaveFormat = nullptr;
    }
    if (device.pAudioClient) {
        device.pAudioClient->Release();
        device.pAudioClient = nullptr;
    }
    if (device.pDevice) {
        device.pDevice->Release();
        device.pDevice = nullptr;
    }
}

void LameRecorder::ApplyVolume(float* samples, int frameCount, int volumePercent) {
    float volumeMultiplier = volumePercent / 100.0f;
    int sampleCount = frameCount * 2; // 立体声
    for (int i = 0; i < sampleCount; i++) {
        samples[i] *= volumeMultiplier;
    }
}

void LameRecorder::MixAudioSamples(const float* inputSamples, int inputFrames,
    const float* outputSamples, int outputFrames,
    short* mixedSamples, int& mixedFrames) {
    mixedFrames = (inputFrames < outputFrames) ? inputFrames : outputFrames;

    for (int i = 0; i < mixedFrames * 2; i++) {  // *2 for stereo
        float mixed = (inputSamples[i] + outputSamples[i]) * 0.5f;  // 简单平均混音

        // 限幅
        if (mixed > 1.0f) mixed = 1.0f;
        if (mixed < -1.0f) mixed = -1.0f;

        // 转换为16位整数
        mixedSamples[i] = (short)(mixed * 32767.0f);
    }
}

void LameRecorder::RecordThreadProc() {
    WriteLog(L"[LameRecorder] 录音线程启动");

    HANDLE waitEvents[2] = {
        m_inputDevice.hAudioSamplesReadyEvent,
        m_outputDevice.hAudioSamplesReadyEvent
    };

    float inputBuffer[BUFFER_SIZE] = { 0 };
    float outputBuffer[BUFFER_SIZE] = { 0 };
    short pcmBuffer[BUFFER_SIZE * 2];
    unsigned char mp3Buffer[BUFFER_SIZE * 2];

    UINT32 lastInputFrames = 0;
    UINT32 lastOutputFrames = 0;

    while (m_recording) {
        // 等待音频数据
        DWORD waitResult = WaitForMultipleObjects(2, waitEvents, FALSE, 100);

        // 重置缓冲区（避免使用上一次的数据）
        UINT32 inputFramesAvailable = 0;
        UINT32 outputFramesAvailable = 0;

        // 从输入设备读取数据
        BYTE* pInputData = nullptr;
        DWORD inputFlags = 0;

        HRESULT hr = m_inputDevice.pCaptureClient->GetBuffer(
            &pInputData, &inputFramesAvailable, &inputFlags, NULL, NULL);

        if (SUCCEEDED(hr) && inputFramesAvailable > 0) {
            // 检查是否静音标志
            if (inputFlags & AUDCLNT_BUFFERFLAGS_SILENT) {
                // 静音，填充0
                memset(inputBuffer, 0, inputFramesAvailable * m_inputDevice.pWaveFormat->nBlockAlign);
            }
            else {
                memcpy(inputBuffer, pInputData, inputFramesAvailable * m_inputDevice.pWaveFormat->nBlockAlign);
                ApplyVolume(inputBuffer, inputFramesAvailable, m_micVolume);
            }
            m_inputDevice.pCaptureClient->ReleaseBuffer(inputFramesAvailable);
            lastInputFrames = inputFramesAvailable;
        }
        else if (lastInputFrames > 0) {
            // 如果这次没有数据，但之前有，使用静音数据
            inputFramesAvailable = lastInputFrames;
            memset(inputBuffer, 0, inputFramesAvailable * sizeof(float) * 2);
        }

        // 从输出设备读取数据
        BYTE* pOutputData = nullptr;
        DWORD outputFlags = 0;

        hr = m_outputDevice.pCaptureClient->GetBuffer(
            &pOutputData, &outputFramesAvailable, &outputFlags, NULL, NULL);

        if (SUCCEEDED(hr) && outputFramesAvailable > 0) {
            // 检查是否静音标志
            if (outputFlags & AUDCLNT_BUFFERFLAGS_SILENT) {
                memset(outputBuffer, 0, outputFramesAvailable * m_outputDevice.pWaveFormat->nBlockAlign);
            }
            else {
                memcpy(outputBuffer, pOutputData, outputFramesAvailable * m_outputDevice.pWaveFormat->nBlockAlign);
                ApplyVolume(outputBuffer, outputFramesAvailable, m_speakerVolume);
            }
            m_outputDevice.pCaptureClient->ReleaseBuffer(outputFramesAvailable);
            lastOutputFrames = outputFramesAvailable;
        }
        else if (lastOutputFrames > 0) {
            // 如果这次没有数据，但之前有，使用静音数据
            outputFramesAvailable = lastOutputFrames;
            memset(outputBuffer, 0, outputFramesAvailable * sizeof(float) * 2);
        }

        // 混音并编码（只要有任一设备有数据就处理）
        int mixedFrames = 0;
        if (inputFramesAvailable > 0 || outputFramesAvailable > 0) {
            // 使用较大的帧数
            UINT32 framesToProcess = (inputFramesAvailable > outputFramesAvailable) ?
                inputFramesAvailable : outputFramesAvailable;

            // 如果某个设备没有数据，用静音填充
            if (inputFramesAvailable == 0) {
                memset(inputBuffer, 0, framesToProcess * sizeof(float) * 2);
                inputFramesAvailable = framesToProcess;
            }
            if (outputFramesAvailable == 0) {
                memset(outputBuffer, 0, framesToProcess * sizeof(float) * 2);
                outputFramesAvailable = framesToProcess;
            }

            MixAudioSamples(inputBuffer, inputFramesAvailable,
                outputBuffer, outputFramesAvailable,
                pcmBuffer, mixedFrames);

            // 编码为MP3
            if (mixedFrames > 0) {
                int mp3Bytes = lame_encode_buffer_interleaved(
                    m_lame,
                    pcmBuffer,
                    mixedFrames,
                    mp3Buffer,
                    sizeof(mp3Buffer));

                if (mp3Bytes > 0) {
                    fwrite(mp3Buffer, 1, mp3Bytes, m_outputFile);
                    m_fileSize += mp3Bytes;
                    fflush(m_outputFile);  // 确保数据写入磁盘
                }
                else if (mp3Bytes < 0) {
                    WriteLog(L"[LameRecorder] LAME编码错误: %d", mp3Bytes);
                }
            }
        }

        // 更新录音时长
        m_duration = (GetTickCount() - m_startTime) / 1000;
    }

    WriteLog(L"[LameRecorder] 录音线程结束");
}