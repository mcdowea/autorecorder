// 双通道录音模块
// 同时捕获麦克风输入和扬声器输出

use anyhow::{Context, Result};
use crossbeam_channel::{bounded, Sender, Receiver};
use std::sync::Arc;
use parking_lot::Mutex;
use std::time::{Duration, Instant};
use std::path::PathBuf;

#[cfg(target_os = "windows")]
use windows::{
    core::*,
    Win32::Media::Audio::*,
    Win32::System::Com::*,
    Win32::Foundation::*,
};

pub struct DualChannelRecorder {
    sample_rate: u32,
    mic_gain: f32,      // 麦克风增益 (0.0 - 2.0)
    speaker_gain: f32,  // 扬声器增益 (0.0 - 2.0)
    running: Arc<Mutex<bool>>,
}

pub struct RecordingSession {
    pub mic_receiver: Receiver<Vec<f32>>,
    pub speaker_receiver: Receiver<Vec<f32>>,
    pub stop_signal: Arc<Mutex<bool>>,
}

impl DualChannelRecorder {
    pub fn new(sample_rate: u32) -> Self {
        Self {
            sample_rate,
            mic_gain: 1.0,
            speaker_gain: 1.0,
            running: Arc::new(Mutex::new(false)),
        }
    }

    pub fn set_mic_gain(&mut self, gain: f32) {
        self.mic_gain = gain.clamp(0.0, 2.0);
    }

    pub fn set_speaker_gain(&mut self, gain: f32) {
        self.speaker_gain = gain.clamp(0.0, 2.0);
    }

    #[cfg(target_os = "windows")]
    pub fn start_recording(&self) -> Result<RecordingSession> {
        let (mic_tx, mic_rx) = bounded(1000);
        let (speaker_tx, speaker_rx) = bounded(1000);
        let stop_signal = Arc::new(Mutex::new(false));

        let mic_gain = self.mic_gain;
        let speaker_gain = self.speaker_gain;
        let sample_rate = self.sample_rate;

        // 启动麦克风录音线程
        let stop_signal_mic = Arc::clone(&stop_signal);
        std::thread::spawn(move || {
            if let Err(e) = Self::capture_microphone(mic_tx, mic_gain, sample_rate, stop_signal_mic) {
                eprintln!("Microphone capture error: {}", e);
            }
        });

        // 启动扬声器录音线程(WASAPI Loopback)
        let stop_signal_speaker = Arc::clone(&stop_signal);
        std::thread::spawn(move || {
            if let Err(e) = Self::capture_speaker(speaker_tx, speaker_gain, sample_rate, stop_signal_speaker) {
                eprintln!("Speaker capture error: {}", e);
            }
        });

        Ok(RecordingSession {
            mic_receiver: mic_rx,
            speaker_receiver: speaker_rx,
            stop_signal,
        })
    }

    #[cfg(target_os = "windows")]
    fn capture_microphone(
        tx: Sender<Vec<f32>>,
        gain: f32,
        target_sample_rate: u32,
        stop_signal: Arc<Mutex<bool>>,
    ) -> Result<()> {
        unsafe {
            CoInitializeEx(None, COINIT_MULTITHREADED)?;

            let enumerator: IMMDeviceEnumerator = CoCreateInstance(
                &MMDeviceEnumerator,
                None,
                CLSCTX_ALL,
            )?;

            // 获取默认麦克风设备
            let device = enumerator.GetDefaultAudioEndpoint(eCapture, eConsole)?;

            let audio_client: IAudioClient = device.Activate(CLSCTX_ALL, None)?;
            let pwfx = audio_client.GetMixFormat()?;
            let wave_format = *pwfx;

            // 初始化音频客户端
            audio_client.Initialize(
                AUDCLNT_SHAREMODE_SHARED,
                AUDCLNT_STREAMFLAGS_EVENTCALLBACK,
                10000000,
                0,
                pwfx,
                None,
            )?;

            let capture_client: IAudioCaptureClient = audio_client.GetService()?;
            audio_client.Start()?;

            println!("麦克风录音启动: {} Hz, {} channels, gain: {:.1}x", 
                wave_format.nSamplesPerSec, 
                wave_format.nChannels,
                gain);

            // 捕获循环
            while !*stop_signal.lock() {
                std::thread::sleep(Duration::from_millis(10));

                let packet_length = capture_client.GetNextPacketSize()?;
                if packet_length == 0 {
                    continue;
                }

                let mut data_ptr = std::ptr::null_mut();
                let mut num_frames = 0u32;
                let mut flags = 0u32;

                capture_client.GetBuffer(
                    &mut data_ptr,
                    &mut num_frames,
                    &mut flags,
                    None,
                    None,
                )?;

                if num_frames > 0 && !data_ptr.is_null() {
                    let samples = Self::process_audio_buffer(
                        data_ptr,
                        num_frames as usize,
                        &wave_format,
                        gain,
                    );
                    let _ = tx.try_send(samples);
                }

                capture_client.ReleaseBuffer(num_frames)?;
            }

            audio_client.Stop()?;
            CoUninitialize();
        }

        Ok(())
    }

    #[cfg(target_os = "windows")]
    fn capture_speaker(
        tx: Sender<Vec<f32>>,
        gain: f32,
        target_sample_rate: u32,
        stop_signal: Arc<Mutex<bool>>,
    ) -> Result<()> {
        unsafe {
            CoInitializeEx(None, COINIT_MULTITHREADED)?;

            let enumerator: IMMDeviceEnumerator = CoCreateInstance(
                &MMDeviceEnumerator,
                None,
                CLSCTX_ALL,
            )?;

            // 获取默认扬声器设备
            let device = enumerator.GetDefaultAudioEndpoint(eRender, eConsole)?;

            let audio_client: IAudioClient = device.Activate(CLSCTX_ALL, None)?;
            let pwfx = audio_client.GetMixFormat()?;
            let wave_format = *pwfx;

            // 初始化为Loopback模式
            audio_client.Initialize(
                AUDCLNT_SHAREMODE_SHARED,
                AUDCLNT_STREAMFLAGS_LOOPBACK,  // Loopback标志
                10000000,
                0,
                pwfx,
                None,
            )?;

            let capture_client: IAudioCaptureClient = audio_client.GetService()?;
            audio_client.Start()?;

            println!("扬声器录音启动(Loopback): {} Hz, {} channels, gain: {:.1}x", 
                wave_format.nSamplesPerSec, 
                wave_format.nChannels,
                gain);

            // 捕获循环
            while !*stop_signal.lock() {
                std::thread::sleep(Duration::from_millis(10));

                let packet_length = capture_client.GetNextPacketSize()?;
                if packet_length == 0 {
                    continue;
                }

                let mut data_ptr = std::ptr::null_mut();
                let mut num_frames = 0u32;
                let mut flags = 0u32;

                capture_client.GetBuffer(
                    &mut data_ptr,
                    &mut num_frames,
                    &mut flags,
                    None,
                    None,
                )?;

                if num_frames > 0 && !data_ptr.is_null() {
                    let samples = Self::process_audio_buffer(
                        data_ptr,
                        num_frames as usize,
                        &wave_format,
                        gain,
                    );
                    let _ = tx.try_send(samples);
                }

                capture_client.ReleaseBuffer(num_frames)?;
            }

            audio_client.Stop()?;
            CoUninitialize();
        }

        Ok(())
    }

    #[cfg(target_os = "windows")]
    unsafe fn process_audio_buffer(
        data_ptr: *mut u8,
        num_frames: usize,
        wave_format: &WAVEFORMATEX,
        gain: f32,
    ) -> Vec<f32> {
        let channels = wave_format.nChannels as usize;
        let bits_per_sample = wave_format.wBitsPerSample;

        let mut samples = Vec::new();

        match bits_per_sample {
            16 => {
                let data = std::slice::from_raw_parts(
                    data_ptr as *const i16,
                    num_frames * channels,
                );

                for chunk in data.chunks(channels) {
                    let sum: f32 = chunk.iter()
                        .map(|&s| (s as f32 / 32768.0) * gain)
                        .sum();
                    samples.push(sum / channels as f32);
                }
            }
            32 => {
                let data = std::slice::from_raw_parts(
                    data_ptr as *const f32,
                    num_frames * channels,
                );

                for chunk in data.chunks(channels) {
                    let sum: f32 = chunk.iter()
                        .map(|&s| s * gain)
                        .sum();
                    samples.push(sum / channels as f32);
                }
            }
            _ => {
                eprintln!("Unsupported bit depth: {}", bits_per_sample);
            }
        }

        samples
    }
}

// 音频混音器
pub struct AudioMixer {
    mic_buffer: Vec<f32>,
    speaker_buffer: Vec<f32>,
}

impl AudioMixer {
    pub fn new() -> Self {
        Self {
            mic_buffer: Vec::new(),
            speaker_buffer: Vec::new(),
        }
    }

    pub fn add_mic_samples(&mut self, samples: Vec<f32>) {
        self.mic_buffer.extend(samples);
    }

    pub fn add_speaker_samples(&mut self, samples: Vec<f32>) {
        self.speaker_buffer.extend(samples);
    }

    /// 混合两个通道的音频数据
    pub fn mix(&mut self) -> Vec<f32> {
        let min_len = self.mic_buffer.len().min(self.speaker_buffer.len());
        
        let mixed: Vec<f32> = (0..min_len)
            .map(|i| {
                // 简单混音: 平均两个通道
                (self.mic_buffer[i] + self.speaker_buffer[i]) / 2.0
            })
            .collect();

        // 清空已混音的数据
        self.mic_buffer.drain(..min_len);
        self.speaker_buffer.drain(..min_len);

        mixed
    }

    /// 获取缓冲区大小(用于检查是否有足够数据)
    pub fn buffer_size(&self) -> (usize, usize) {
        (self.mic_buffer.len(), self.speaker_buffer.len())
    }

    /// 清空缓冲区
    pub fn clear(&mut self) {
        self.mic_buffer.clear();
        self.speaker_buffer.clear();
    }
}

#[cfg(not(target_os = "windows"))]
impl DualChannelRecorder {
    pub fn start_recording(&self) -> Result<RecordingSession> {
        Err(anyhow::anyhow!("Only supported on Windows"))
    }
}
