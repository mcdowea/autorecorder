// 智能录音控制器
// 整合麦克风检测、双通道录音和MP3编码

use anyhow::Result;
use std::path::PathBuf;
use std::sync::Arc;
use parking_lot::Mutex;
use std::time::{Duration, Instant};
use chrono::Local;

use crate::mic_detector::MicrophoneDetector;
use crate::dual_recorder::{DualChannelRecorder, AudioMixer};
use crate::mp3_encoder::{StreamingMp3Encoder, WavEncoder};

#[derive(Debug, Clone)]
pub struct RecorderConfig {
    pub output_dir: PathBuf,
    pub sample_rate: u32,
    pub bit_rate: u32,
    pub quality: u8,
    pub mic_gain: f32,
    pub speaker_gain: f32,
    pub blacklist: Vec<String>,
    pub auto_create_folders: bool,
    pub save_format: AudioFormat,
    pub min_recording_duration: Duration,  // 最小录音时长(避免误触发)
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AudioFormat {
    Mp3,
    Wav,
}

impl Default for RecorderConfig {
    fn default() -> Self {
        Self {
            output_dir: PathBuf::from("recordings"),
            sample_rate: 48000,
            bit_rate: 128,
            quality: 2,
            mic_gain: 1.0,
            speaker_gain: 1.0,
            blacklist: vec![
                "chrome.exe".to_string(),
                "firefox.exe".to_string(),
                "msedge.exe".to_string(),
            ],
            auto_create_folders: true,
            save_format: AudioFormat::Mp3,
            min_recording_duration: Duration::from_secs(3),
        }
    }
}

pub struct SmartRecorder {
    config: RecorderConfig,
    detector: MicrophoneDetector,
    is_recording: Arc<Mutex<bool>>,
    current_session: Arc<Mutex<Option<String>>>,  // 当前录音的应用名称
}

impl SmartRecorder {
    pub fn new(config: RecorderConfig) -> Self {
        let mut detector = MicrophoneDetector::new();
        detector.set_blacklist(config.blacklist.clone());

        Self {
            config,
            detector,
            is_recording: Arc::new(Mutex::new(false)),
            current_session: Arc::new(Mutex::new(None)),
        }
    }

    pub fn update_config(&mut self, config: RecorderConfig) {
        self.detector.set_blacklist(config.blacklist.clone());
        self.config = config;
    }

    /// 启动智能监控循环
    pub fn start_monitoring(&mut self) -> Result<()> {
        println!("🎤 智能录音监控已启动");
        println!("📁 录音保存路径: {:?}", self.config.output_dir);
        println!("🚫 进程黑名单: {:?}", self.config.blacklist);

        loop {
            // 检测麦克风占用
            match self.detector.detect_active_sessions() {
                Ok(sessions) => {
                    let active_apps = self.detector.get_active_apps(&sessions);

                    if !active_apps.is_empty() && !*self.is_recording.lock() {
                        // 检测到新的麦克风使用
                        println!("\n✅ 检测到麦克风使用:");
                        for app in &active_apps {
                            println!("   📱 {}", app);
                        }

                        // 开始录音
                        self.start_recording_session(&active_apps[0])?;
                    } else if active_apps.is_empty() && *self.is_recording.lock() {
                        // 麦克风不再被使用
                        println!("\n⏸️  麦克风使用已结束");
                        self.stop_recording_session()?;
                    }
                }
                Err(e) => {
                    eprintln!("检测错误: {}", e);
                }
            }

            std::thread::sleep(Duration::from_secs(1));
        }
    }

    fn start_recording_session(&self, app_name: &str) -> Result<()> {
        *self.is_recording.lock() = true;
        *self.current_session.lock() = Some(app_name.to_string());

        let app_name = app_name.to_string();
        let config = self.config.clone();
        let is_recording = Arc::clone(&self.is_recording);
        let current_session = Arc::clone(&self.current_session);

        // 在新线程中启动录音
        std::thread::spawn(move || {
            if let Err(e) = Self::recording_thread(app_name, config, is_recording, current_session) {
                eprintln!("录音错误: {}", e);
            }
        });

        Ok(())
    }

    fn stop_recording_session(&self) -> Result<()> {
        *self.is_recording.lock() = false;
        *self.current_session.lock() = None;
        Ok(())
    }

    fn recording_thread(
        app_name: String,
        config: RecorderConfig,
        is_recording: Arc<Mutex<bool>>,
        _current_session: Arc<Mutex<Option<String>>>,
    ) -> Result<()> {
        println!("🔴 开始录音...");

        let start_time = Instant::now();

        // 创建双通道录音器
        let mut recorder = DualChannelRecorder::new(config.sample_rate);
        recorder.set_mic_gain(config.mic_gain);
        recorder.set_speaker_gain(config.speaker_gain);

        // 开始录音
        let session = recorder.start_recording()?;

        // 创建音频混音器
        let mut mixer = AudioMixer::new();

        // 创建MP3编码器
        let mut encoder = if config.save_format == AudioFormat::Mp3 {
            Some(StreamingMp3Encoder::new(
                config.sample_rate,
                config.bit_rate,
                config.quality,
            )?)
        } else {
            None
        };

        let mut all_samples = Vec::new();

        // 录音循环
        while *is_recording.lock() {
            // 接收麦克风数据
            while let Ok(samples) = session.mic_receiver.try_recv() {
                mixer.add_mic_samples(samples);
            }

            // 接收扬声器数据
            while let Ok(samples) = session.speaker_receiver.try_recv() {
                mixer.add_speaker_samples(samples);
            }

            // 混音
            let mixed = mixer.mix();
            if !mixed.is_empty() {
                all_samples.extend_from_slice(&mixed);

                // 实时编码(如果使用MP3)
                if let Some(ref mut enc) = encoder {
                    enc.encode_samples(&mixed)?;
                }
            }

            std::thread::sleep(Duration::from_millis(50));
        }

        // 停止录音
        *session.stop_signal.lock() = true;

        let duration = start_time.elapsed();
        println!("⏹️  录音停止 (时长: {:.1}秒)", duration.as_secs_f32());

        // 检查最小时长
        if duration < config.min_recording_duration {
            println!("⚠️  录音时长过短,已丢弃");
            return Ok(());
        }

        // 保存文件
        let output_path = Self::generate_output_path(&config, &app_name);

        match config.save_format {
            AudioFormat::Mp3 => {
                if let Some(enc) = encoder {
                    enc.save_to_file(&output_path)?;
                    println!("💾 录音已保存: {:?} (MP3格式)", output_path);
                }
            }
            AudioFormat::Wav => {
                let wav_encoder = WavEncoder::new(config.sample_rate);
                wav_encoder.encode_to_file(&all_samples, &output_path)?;
                println!("💾 录音已保存: {:?} (WAV格式)", output_path);
            }
        }

        Ok(())
    }

    fn generate_output_path(config: &RecorderConfig, app_name: &str) -> PathBuf {
        let timestamp = Local::now().format("%Y%m%d_%H%M%S");

        // 提取应用名(去除.exe)
        let app_basename = app_name
            .trim_end_matches(".exe")
            .trim_end_matches(".EXE");

        let filename = if config.save_format == AudioFormat::Mp3 {
            format!("{}_{}.mp3", app_basename, timestamp)
        } else {
            format!("{}_{}.wav", app_basename, timestamp)
        };

        if config.auto_create_folders {
            // 创建应用名称文件夹
            let app_folder = config.output_dir.join(app_basename);
            std::fs::create_dir_all(&app_folder).ok();
            app_folder.join(filename)
        } else {
            std::fs::create_dir_all(&config.output_dir).ok();
            config.output_dir.join(filename)
        }
    }

    pub fn is_recording(&self) -> bool {
        *self.is_recording.lock()
    }

    pub fn current_app(&self) -> Option<String> {
        self.current_session.lock().clone()
    }
}
