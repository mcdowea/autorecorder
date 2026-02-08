// 双通道录音模块
// 使用 cpal 库进行跨平台音频录制
// 参考 Deepgram audio-recorder 实现

use anyhow::{Context, Result};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, SampleFormat, StreamConfig};
use crossbeam_channel::{bounded, Sender, Receiver};
use std::sync::Arc;
use parking_lot::Mutex;
use std::time::Duration;

pub struct DualChannelRecorder {
    sample_rate: u32,
    mic_gain: f32,
    speaker_gain: f32,
}

pub struct RecordingSession {
    pub mic_receiver: Receiver<Vec<f32>>,
    pub speaker_receiver: Receiver<Vec<f32>>,
    pub stop_signal: Arc<Mutex<bool>>,
    _mic_stream: cpal::Stream,  // 保持流存活
    _speaker_stream: Option<cpal::Stream>,  // 扬声器流(可选)
}

impl DualChannelRecorder {
    pub fn new(sample_rate: u32) -> Self {
        Self {
            sample_rate,
            mic_gain: 1.0,
            speaker_gain: 1.0,
        }
    }

    pub fn set_mic_gain(&mut self, gain: f32) {
        self.mic_gain = gain.clamp(0.0, 2.0);
    }

    pub fn set_speaker_gain(&mut self, gain: f32) {
        self.speaker_gain = gain.clamp(0.0, 2.0);
    }

    /// 开始录音,返回录音会话
    pub fn start_recording(&self) -> Result<RecordingSession> {
        let host = cpal::default_host();
        
        // 设置通道
        let (mic_tx, mic_rx) = bounded(1000);
        let (speaker_tx, speaker_rx) = bounded(1000);
        let stop_signal = Arc::new(Mutex::new(false));

        // 启动麦克风录音
        let mic_stream = self.start_microphone_capture(&host, mic_tx, self.mic_gain)?;
        
        // 尝试启动扬声器录音 (Loopback)
        // 注意: Windows 上需要使用 WASAPI Loopback
        let speaker_stream = self.start_speaker_capture(&host, speaker_tx, self.speaker_gain).ok();

        if speaker_stream.is_none() {
            println!("Warning: Speaker capture not available on this platform");
        }

        Ok(RecordingSession {
            mic_receiver: mic_rx,
            speaker_receiver: speaker_rx,
            stop_signal,
            _mic_stream: mic_stream,
            _speaker_stream: speaker_stream,
        })
    }

    /// 启动麦克风捕获
    fn start_microphone_capture(
        &self,
        host: &cpal::Host,
        tx: Sender<Vec<f32>>,
        gain: f32,
    ) -> Result<cpal::Stream> {
        let device = host
            .default_input_device()
            .context("No input device available")?;

        let device_name = device.name().unwrap_or_else(|_| "Unknown".to_string());
        println!("🎤 Recording from: {}", device_name);

        let config = device
            .default_input_config()
            .context("Failed to get default input config")?;

        println!("   Config: {} channels, {} Hz, {:?}",
            config.channels(),
            config.sample_rate().0,
            config.sample_format()
        );

        let stream_config: StreamConfig = config.clone().into();

        // 根据样本格式构建流
        let stream = match config.sample_format() {
            SampleFormat::F32 => self.build_input_stream_f32(&device, &stream_config, tx, gain)?,
            SampleFormat::I16 => self.build_input_stream_i16(&device, &stream_config, tx, gain)?,
            SampleFormat::U16 => self.build_input_stream_u16(&device, &stream_config, tx, gain)?,
            _ => return Err(anyhow::anyhow!("Unsupported sample format")),
        };

        stream.play().context("Failed to start microphone stream")?;
        Ok(stream)
    }

    /// 启动扬声器捕获 (Loopback) - Windows only
    #[cfg(target_os = "windows")]
    fn start_speaker_capture(
        &self,
        host: &cpal::Host,
        tx: Sender<Vec<f32>>,
        gain: f32,
    ) -> Result<cpal::Stream> {
        // 在 Windows 上,使用 output device 的 loopback 功能
        // 注意: cpal 本身不直接支持 loopback,需要平台特定的实现
        // 这里我们尝试使用默认输出设备
        
        let device = host
            .default_output_device()
            .context("No output device available")?;

        let device_name = device.name().unwrap_or_else(|_| "Unknown".to_string());
        println!("🔊 Recording from: {} (loopback)", device_name);

        // 注意: cpal 0.16 在 Windows 上使用 WASAPI,但不直接支持 loopback mode
        // 需要使用 Windows API 直接访问或者使用虚拟音频设备
        // 这里我们保留接口,但实际实现可能需要回退到 Windows API
        
        Err(anyhow::anyhow!("Loopback not directly supported via cpal"))
    }

    /// 在非 Windows 平台上,扬声器捕获不可用
    #[cfg(not(target_os = "windows"))]
    fn start_speaker_capture(
        &self,
        _host: &cpal::Host,
        _tx: Sender<Vec<f32>>,
        _gain: f32,
    ) -> Result<cpal::Stream> {
        Err(anyhow::anyhow!("Speaker capture not available on this platform"))
    }

    /// 构建 f32 输入流
    fn build_input_stream_f32(
        &self,
        device: &Device,
        config: &StreamConfig,
        tx: Sender<Vec<f32>>,
        gain: f32,
    ) -> Result<cpal::Stream> {
        let err_fn = |err| eprintln!("Stream error: {}", err);

        let stream = device.build_input_stream(
            config,
            move |data: &[f32], _: &cpal::InputCallbackInfo| {
                // 应用增益并发送样本
                let samples: Vec<f32> = data.iter().map(|&s| s * gain).collect();
                let _ = tx.try_send(samples);
            },
            err_fn,
            None,
        )?;

        Ok(stream)
    }

    /// 构建 i16 输入流
    fn build_input_stream_i16(
        &self,
        device: &Device,
        config: &StreamConfig,
        tx: Sender<Vec<f32>>,
        gain: f32,
    ) -> Result<cpal::Stream> {
        let err_fn = |err| eprintln!("Stream error: {}", err);

        let stream = device.build_input_stream(
            config,
            move |data: &[i16], _: &cpal::InputCallbackInfo| {
                // 转换 i16 到 f32 并应用增益
                let samples: Vec<f32> = data
                    .iter()
                    .map(|&s| (s as f32 / 32768.0) * gain)
                    .collect();
                let _ = tx.try_send(samples);
            },
            err_fn,
            None,
        )?;

        Ok(stream)
    }

    /// 构建 u16 输入流
    fn build_input_stream_u16(
        &self,
        device: &Device,
        config: &StreamConfig,
        tx: Sender<Vec<f32>>,
        gain: f32,
    ) -> Result<cpal::Stream> {
        let err_fn = |err| eprintln!("Stream error: {}", err);

        let stream = device.build_input_stream(
            config,
            move |data: &[u16], _: &cpal::InputCallbackInfo| {
                // 转换 u16 到 f32 (centered at 32768)
                let samples: Vec<f32> = data
                    .iter()
                    .map(|&s| ((s as f32 - 32768.0) / 32768.0) * gain)
                    .collect();
                let _ = tx.try_send(samples);
            },
            err_fn,
            None,
        )?;

        Ok(stream)
    }
}

/// 音频混音器 - 混合麦克风和扬声器音频
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

    /// 添加麦克风样本
    pub fn add_mic_samples(&mut self, samples: Vec<f32>) {
        self.mic_buffer.extend(samples);
    }

    /// 添加扬声器样本
    pub fn add_speaker_samples(&mut self, samples: Vec<f32>) {
        self.speaker_buffer.extend(samples);
    }

    /// 混音并返回混合后的样本
    pub fn mix(&mut self) -> Vec<f32> {
        let len = self.mic_buffer.len().min(self.speaker_buffer.len());
        
        if len == 0 {
            // 如果没有足够的数据混音,返回麦克风数据
            let result = self.mic_buffer.clone();
            self.mic_buffer.clear();
            return result;
        }

        // 混合两个通道
        let mut mixed = Vec::with_capacity(len);
        for i in 0..len {
            let mic = self.mic_buffer[i];
            let speaker = self.speaker_buffer[i];
            // 简单平均混音
            mixed.push((mic + speaker) * 0.5);
        }

        // 移除已混音的样本
        self.mic_buffer.drain(0..len);
        self.speaker_buffer.drain(0..len);

        mixed
    }

    /// 获取剩余的麦克风样本(当没有扬声器数据时使用)
    pub fn flush_mic(&mut self) -> Vec<f32> {
        std::mem::take(&mut self.mic_buffer)
    }
}

impl Default for AudioMixer {
    fn default() -> Self {
        Self::new()
    }
}
