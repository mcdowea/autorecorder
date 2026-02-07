use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use anyhow::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioConfig {
    /// 采样率 (Hz)
    pub sample_rate: u32,
    /// 比特率 (kbps)
    pub bitrate: u32,
    /// 声道数
    pub channels: u16,
    /// MP3 质量 (0-9, 0 最高质量)
    pub quality: u8,
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            sample_rate: 44100,
            bitrate: 128,
            channels: 2,
            quality: 2,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecorderConfig {
    /// 输出目录
    pub output_dir: PathBuf,
    /// 音频配置
    pub audio: AudioConfig,
    /// 监控的程序列表
    pub monitored_apps: Vec<String>,
    /// 是否自动录音
    pub auto_record: bool,
    /// 最小通话时长（秒），少于此时长不保存
    pub min_call_duration: u64,
}

impl Default for RecorderConfig {
    fn default() -> Self {
        let output_dir = dirs::document_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("AudioRecordings");
        
        Self {
            output_dir,
            audio: AudioConfig::default(),
            monitored_apps: vec![
                "WeChat.exe".to_string(),
                "QQ.exe".to_string(),
                "Lark.exe".to_string(),
                "Feishu.exe".to_string(),
                "Skype.exe".to_string(),
                "Teams.exe".to_string(),
                "Zoom.exe".to_string(),
                "DingTalk.exe".to_string(),
            ],
            auto_record: true,
            min_call_duration: 5,
        }
    }
}

impl RecorderConfig {
    pub fn load() -> Result<Self> {
        let config_path = Self::config_path();
        
        if config_path.exists() {
            let content = std::fs::read_to_string(&config_path)?;
            Ok(toml::from_str(&content)?)
        } else {
            let config = Self::default();
            config.save()?;
            Ok(config)
        }
    }
    
    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_path();
        
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        
        let content = toml::to_string_pretty(self)?;
        std::fs::write(config_path, content)?;
        Ok(())
    }
    
    fn config_path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("auto-audio-recorder")
            .join("config.toml")
    }
    
    pub fn ensure_output_dir(&self) -> Result<()> {
        std::fs::create_dir_all(&self.output_dir)?;
        Ok(())
    }
}
