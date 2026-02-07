use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// 录音输出目录
    pub output_dir: PathBuf,
    
    /// 采样率 (Hz)
    pub sample_rate: u32,
    
    /// 比特率 (kbps) - 保留用于将来MP3支持
    pub bit_rate: u32,
    
    /// 质量 (0-9, 0最好9最差) - 保留用于将来MP3支持
    pub quality: u8,
    
    /// 是否启用自动录音
    pub auto_recording: bool,
    
    /// 监控的通话应用程序
    pub monitored_apps: Vec<String>,
    
    /// 静音检测阈值 (0.0-1.0)
    pub silence_threshold: f32,
    
    /// 静音持续时间停止录音 (秒)
    pub silence_duration: u64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            output_dir: PathBuf::from("recordings"),
            sample_rate: 44100,
            bit_rate: 128,
            quality: 2,
            auto_recording: true,
            monitored_apps: vec![
                "WeChat.exe".to_string(),
                "QQ.exe".to_string(),
                "Lark.exe".to_string(),
                "feishu.exe".to_string(),
                "Skype.exe".to_string(),
                "Teams.exe".to_string(),
                "Zoom.exe".to_string(),
                "Discord.exe".to_string(),
            ],
            silence_threshold: 0.01,
            silence_duration: 3,
        }
    }
}

impl Config {
    pub fn load_or_default(path: &PathBuf) -> anyhow::Result<Self> {
        if path.exists() {
            let content = std::fs::read_to_string(path)?;
            Ok(serde_json::from_str(&content)?)
        } else {
            let config = Self::default();
            config.save(path)?;
            Ok(config)
        }
    }

    pub fn save(&self, path: &PathBuf) -> anyhow::Result<()> {
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }
}
