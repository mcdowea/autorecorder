use crate::audio_capture::{AudioCapture, AudioSource};
use crate::config::Config;
use crate::mp3_encoder::Mp3Encoder;
use crate::process_monitor::ProcessMonitor;
use anyhow::{Context, Result};
use chrono::Local;
use cpal::traits::StreamTrait;
use crossbeam_channel::Receiver;
use std::path::PathBuf;
use std::sync::Arc;
use parking_lot::Mutex;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RecordingState {
    Idle,
    Recording,
    Paused,
}

pub struct Recorder {
    config: Config,
    state: Arc<Mutex<RecordingState>>,
    process_monitor: ProcessMonitor,
}

impl Recorder {
    pub fn new(config: Config) -> Self {
        let process_monitor = ProcessMonitor::new(config.monitored_apps.clone());
        
        Self {
            config,
            state: Arc::new(Mutex::new(RecordingState::Idle)),
            process_monitor,
        }
    }

    pub fn get_state(&self) -> RecordingState {
        *self.state.lock()
    }

    pub fn start_manual_recording(&self) -> Result<()> {
        tracing::info!("Starting manual recording...");
        self.record_session(false)
    }

    pub fn start_auto_monitoring(&self) -> Result<()> {
        tracing::info!("Starting auto monitoring mode...");
        
        loop {
            if self.process_monitor.is_call_active() {
                tracing::info!("Call detected! Starting recording...");
                *self.state.lock() = RecordingState::Recording;
                
                if let Err(e) = self.record_session(true) {
                    tracing::error!("Recording session error: {}", e);
                }
                
                *self.state.lock() = RecordingState::Idle;
                tracing::info!("Call ended. Waiting for next call...");
            }
            
            std::thread::sleep(Duration::from_secs(2));
        }
    }

    fn record_session(&self, auto_mode: bool) -> Result<()> {
        // 确保输出目录存在
        std::fs::create_dir_all(&self.config.output_dir)?;

        // 生成文件名
        let timestamp = Local::now().format("%Y%m%d_%H%M%S");
        let filename = format!("recording_{}.mp3", timestamp);
        let output_path = self.config.output_dir.join(filename);

        tracing::info!("Recording to: {:?}", output_path);

        // 初始化音频捕获
        let mut audio_capture = AudioCapture::new()?;
        audio_capture.init_microphone()?;
        audio_capture.init_speaker()?;

        // 创建音频流
        let (mic_stream, mic_rx) = audio_capture.create_stream(
            AudioSource::Microphone,
            self.config.sample_rate,
        )?;

        let (speaker_stream, speaker_rx) = audio_capture.create_stream(
            AudioSource::Speaker,
            self.config.sample_rate,
        )?;

        // 启动流
        mic_stream.play()?;
        speaker_stream.play()?;

        // 创建 MP3 编码器
        let mut encoder = Mp3Encoder::new(
            &output_path,
            self.config.sample_rate,
            self.config.bit_rate,
            self.config.quality,
        )?;

        // 录音循环
        let mut silence_start: Option<Instant> = None;
        let mut total_samples = 0u64;

        tracing::info!("Recording started. Press Ctrl+C to stop (manual mode)");

        loop {
            // 在自动模式下检查通话是否仍在进行
            if auto_mode && !self.process_monitor.is_call_active() {
                tracing::info!("Call ended, stopping recording...");
                break;
            }

            // 混合麦克风和扬声器音频
            let mut mixed_samples = Vec::new();
            let mut has_audio = false;

            // 从麦克风接收
            while let Ok(samples) = mic_rx.try_recv() {
                if mixed_samples.is_empty() {
                    mixed_samples = samples;
                } else {
                    // 混合音频
                    for (i, &sample) in samples.iter().enumerate() {
                        if i < mixed_samples.len() {
                            mixed_samples[i] = (mixed_samples[i] + sample) / 2.0;
                        } else {
                            mixed_samples.push(sample);
                        }
                    }
                }
                has_audio = true;
            }

            // 从扬声器接收
            while let Ok(samples) = speaker_rx.try_recv() {
                if mixed_samples.is_empty() {
                    mixed_samples = samples;
                } else {
                    // 混合音频
                    for (i, &sample) in samples.iter().enumerate() {
                        if i < mixed_samples.len() {
                            mixed_samples[i] = (mixed_samples[i] + sample) / 2.0;
                        } else {
                            mixed_samples.push(sample);
                        }
                    }
                }
                has_audio = true;
            }

            if !mixed_samples.is_empty() {
                // 检测静音
                let max_amplitude = mixed_samples.iter()
                    .map(|&s| s.abs())
                    .fold(0.0f32, f32::max);

                if max_amplitude < self.config.silence_threshold {
                    if silence_start.is_none() {
                        silence_start = Some(Instant::now());
                    } else if let Some(start) = silence_start {
                        if start.elapsed() > Duration::from_secs(self.config.silence_duration) {
                            if !auto_mode {
                                tracing::info!("Silence detected for {} seconds, stopping...", 
                                    self.config.silence_duration);
                                break;
                            }
                        }
                    }
                } else {
                    silence_start = None;
                }

                // 编码样本
                encoder.encode_samples(&mixed_samples)?;
                total_samples += mixed_samples.len() as u64;

                // 每5秒输出一次进度
                if total_samples % (self.config.sample_rate as u64 * 5) < mixed_samples.len() as u64 {
                    let duration_secs = total_samples / self.config.sample_rate as u64;
                    tracing::info!("Recording... {}:{:02}", duration_secs / 60, duration_secs % 60);
                }
            }

            if !has_audio {
                std::thread::sleep(Duration::from_millis(10));
            }
        }

        // 完成编码
        drop(mic_stream);
        drop(speaker_stream);
        encoder.finish()?;

        let duration_secs = total_samples / self.config.sample_rate as u64;
        tracing::info!(
            "Recording completed: {:?} (duration: {}:{:02})",
            output_path,
            duration_secs / 60,
            duration_secs % 60
        );

        Ok(())
    }
}
