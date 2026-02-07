use crate::audio_capture::{AudioCapture, windows_loopback};
use crate::config::{AudioConfig, RecorderConfig};
use crate::encoder::Mp3Encoder;
use anyhow::Result;
use chrono::Local;
use cpal::traits::StreamTrait;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use tokio::sync::RwLock;
use tracing::{info, warn, error};

#[derive(Debug, Clone, PartialEq)]
pub enum RecordingState {
    Idle,
    Recording,
    Stopped,
}

pub struct Recorder {
    config: RecorderConfig,
    state: Arc<RwLock<RecordingState>>,
    mic_buffer: Arc<Mutex<Vec<f32>>>,
    speaker_buffer: Arc<Mutex<Vec<f32>>>,
    start_time: Arc<Mutex<Option<Instant>>>,
}

impl Recorder {
    pub fn new(config: RecorderConfig) -> Result<Self> {
        config.ensure_output_dir()?;
        
        Ok(Self {
            config,
            state: Arc::new(RwLock::new(RecordingState::Idle)),
            mic_buffer: Arc::new(Mutex::new(Vec::new())),
            speaker_buffer: Arc::new(Mutex::new(Vec::new())),
            start_time: Arc::new(Mutex::new(None)),
        })
    }
    
    pub async fn get_state(&self) -> RecordingState {
        self.state.read().await.clone()
    }
    
    /// 开始录音
    pub async fn start_recording(&self) -> Result<()> {
        let mut state = self.state.write().await;
        
        if *state == RecordingState::Recording {
            warn!("录音已在进行中");
            return Ok(());
        }
        
        info!("开始录音...");
        
        // 清空缓冲区
        self.mic_buffer.lock().unwrap().clear();
        self.speaker_buffer.lock().unwrap().clear();
        *self.start_time.lock().unwrap() = Some(Instant::now());
        
        *state = RecordingState::Recording;
        
        Ok(())
    }
    
    /// 停止录音并保存
    pub async fn stop_recording(&self) -> Result<Option<PathBuf>> {
        let mut state = self.state.write().await;
        
        if *state != RecordingState::Recording {
            warn!("当前没有正在进行的录音");
            return Ok(None);
        }
        
        info!("停止录音...");
        *state = RecordingState::Stopped;
        
        // 获取录音时长
        let duration = self.start_time
            .lock()
            .unwrap()
            .map(|t| t.elapsed().as_secs())
            .unwrap_or(0);
        
        // 检查最小时长
        if duration < self.config.min_call_duration {
            info!("录音时长 {} 秒，少于最小时长 {} 秒，不保存", 
                  duration, self.config.min_call_duration);
            *state = RecordingState::Idle;
            return Ok(None);
        }
        
        // 保存录音
        let output_path = self.save_recording().await?;
        
        *state = RecordingState::Idle;
        
        Ok(Some(output_path))
    }
    
    /// 保存录音到文件
    async fn save_recording(&self) -> Result<PathBuf> {
        let mic_data = self.mic_buffer.lock().unwrap().clone();
        let speaker_data = self.speaker_buffer.lock().unwrap().clone();
        
        info!("麦克风数据: {} 样本, 扬声器数据: {} 样本", 
              mic_data.len(), speaker_data.len());
        
        if mic_data.is_empty() && speaker_data.is_empty() {
            return Err(anyhow::anyhow!("没有可保存的音频数据"));
        }
        
        // 混合音频
        let mixed_data = Mp3Encoder::mix_channels(&mic_data, &speaker_data);
        
        // 生成文件名
        let timestamp = Local::now().format("%Y%m%d_%H%M%S");
        let filename = format!("recording_{}.mp3", timestamp);
        let output_path = self.config.output_dir.join(filename);
        
        // 编码为 MP3
        let encoder = Mp3Encoder::new(
            self.config.audio.sample_rate,
            self.config.audio.channels,
            self.config.audio.bitrate,
            self.config.audio.quality,
        );
        
        encoder.encode_to_file(&mixed_data, &output_path)?;
        
        info!("录音已保存到: {:?}", output_path);
        
        Ok(output_path)
    }
    
    /// 运行录音器（捕获音频）
    pub async fn run(
        &self,
        mut shutdown: tokio::sync::watch::Receiver<bool>,
    ) -> Result<()> {
        let audio_capture = AudioCapture::new()?;
        
        // 获取设备
        let mic_device = audio_capture.get_input_device()?;
        let mic_config = audio_capture.get_device_config(&mic_device)?;
        
        info!("麦克风设备: {:?}", mic_device.name());
        
        // 创建麦克风流
        let mic_stream = audio_capture.create_capture_stream(
            &mic_device,
            &mic_config,
            self.config.audio.sample_rate,
            Arc::clone(&self.mic_buffer),
        )?;
        
        mic_stream.play()?;
        info!("麦克风流已启动");
        
        // Windows 下捕获扬声器
        #[cfg(target_os = "windows")]
        let speaker_stream = {
            match windows_loopback::get_loopback_device() {
                Ok(device) => {
                    info!("扬声器设备: {:?}", device.name());
                    match windows_loopback::create_loopback_stream(
                        &device,
                        Arc::clone(&self.speaker_buffer),
                    ) {
                        Ok(stream) => {
                            stream.play()?;
                            info!("扬声器 Loopback 流已启动");
                            Some(stream)
                        }
                        Err(e) => {
                            warn!("无法创建扬声器捕获流: {}", e);
                            None
                        }
                    }
                }
                Err(e) => {
                    warn!("无法获取扬声器设备: {}", e);
                    None
                }
            }
        };
        
        // 等待关闭信号
        let _ = shutdown.changed().await;
        
        info!("音频捕获已停止");
        
        Ok(())
    }
}

pub struct RecorderManager {
    recorder: Arc<Recorder>,
    capture_handle: Option<tokio::task::JoinHandle<Result<()>>>,
    shutdown_tx: Option<tokio::sync::watch::Sender<bool>>,
}

impl RecorderManager {
    pub fn new(config: RecorderConfig) -> Result<Self> {
        let recorder = Arc::new(Recorder::new(config)?);
        
        Ok(Self {
            recorder,
            capture_handle: None,
            shutdown_tx: None,
        })
    }
    
    /// 启动录音器
    pub async fn start(&mut self) -> Result<()> {
        if self.capture_handle.is_some() {
            warn!("录音器已在运行");
            return Ok(());
        }
        
        let (tx, rx) = tokio::sync::watch::channel(false);
        self.shutdown_tx = Some(tx);
        
        let recorder = Arc::clone(&self.recorder);
        let handle = tokio::spawn(async move {
            recorder.run(rx).await
        });
        
        self.capture_handle = Some(handle);
        
        Ok(())
    }
    
    /// 停止录音器
    pub async fn stop(&mut self) -> Result<()> {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(true);
        }
        
        if let Some(handle) = self.capture_handle.take() {
            handle.await??;
        }
        
        Ok(())
    }
    
    /// 开始录音
    pub async fn start_recording(&self) -> Result<()> {
        self.recorder.start_recording().await
    }
    
    /// 停止录音
    pub async fn stop_recording(&self) -> Result<Option<PathBuf>> {
        self.recorder.stop_recording().await
    }
    
    /// 获取录音状态
    pub async fn get_state(&self) -> RecordingState {
        self.recorder.get_state().await
    }
}
