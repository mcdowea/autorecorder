mod audio_capture;
mod config;
mod mp3_encoder;
mod process_monitor;
mod recorder;

use anyhow::Result;
use clap::{Parser, Subcommand};
use config::Config;
use recorder::Recorder;
use std::path::PathBuf;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Parser)]
#[command(name = "auto-recorder")]
#[command(author = "Auto Recorder Team")]
#[command(version = "0.1.0")]
#[command(about = "Automatic audio recorder for microphone and speakers", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// 配置文件路径
    #[arg(short, long, default_value = "config.json")]
    config: PathBuf,

    /// 启用详细日志
    #[arg(short, long)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// 启动自动录音监控
    Auto,
    
    /// 手动开始录音
    Record {
        /// 采样率 (Hz)
        #[arg(short, long)]
        sample_rate: Option<u32>,
        
        /// 比特率 (kbps)
        #[arg(short, long)]
        bit_rate: Option<u32>,
        
        /// 质量 (0-9, 0最好)
        #[arg(short, long)]
        quality: Option<u8>,
        
        /// 输出目录
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    
    /// 列出所有音频设备
    ListDevices,
    
    /// 生成默认配置文件
    GenConfig,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // 初始化日志
    let log_level = if cli.verbose { "debug" } else { "info" };
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| format!("auto_recorder={}", log_level).into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Auto Recorder v0.1.0");
    tracing::info!("==================");

    match cli.command {
        Commands::Auto => {
            let config = Config::load_or_default(&cli.config)?;
            tracing::info!("Loaded configuration from: {:?}", cli.config);
            tracing::info!("Output directory: {:?}", config.output_dir);
            tracing::info!("Monitored apps: {:?}", config.monitored_apps);
            
            let recorder = Recorder::new(config);
            recorder.start_auto_monitoring()?;
        }

        Commands::Record {
            sample_rate,
            bit_rate,
            quality,
            output,
        } => {
            let mut config = Config::load_or_default(&cli.config)?;
            
            if let Some(sr) = sample_rate {
                config.sample_rate = sr;
            }
            if let Some(br) = bit_rate {
                config.bit_rate = br;
            }
            if let Some(q) = quality {
                config.quality = q.min(9);
            }
            if let Some(out) = output {
                config.output_dir = out;
            }

            tracing::info!("Manual recording mode");
            tracing::info!("Sample rate: {} Hz", config.sample_rate);
            tracing::info!("Bit rate: {} kbps", config.bit_rate);
            tracing::info!("Quality: {}/9", config.quality);
            tracing::info!("Output directory: {:?}", config.output_dir);

            let recorder = Recorder::new(config);
            
            // 设置 Ctrl+C 处理
            ctrlc::set_handler(move || {
                tracing::info!("\nReceived Ctrl+C, stopping recording...");
                std::process::exit(0);
            })?;

            recorder.start_manual_recording()?;
        }

        Commands::ListDevices => {
            tracing::info!("Listing audio devices...\n");
            let mut capture = audio_capture::AudioCapture::new()?;
            capture.list_devices()?;
        }

        Commands::GenConfig => {
            let config = Config::default();
            config.save(&cli.config)?;
            tracing::info!("Generated default configuration: {:?}", cli.config);
            tracing::info!("\nConfiguration:");
            tracing::info!("{}", serde_json::to_string_pretty(&config)?);
        }
    }

    Ok(())
}
