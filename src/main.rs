mod mic_detector;
mod dual_recorder;
mod mp3_encoder;
mod smart_recorder;

use anyhow::Result;
use clap::{Parser, Subcommand};
use smart_recorder::{SmartRecorder, RecorderConfig};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "smart-recorder")]
#[command(author = "Smart Recorder Team")]
#[command(version = "1.0.0")]
#[command(about = "智能录音工具 - 自动检测麦克风使用并录音", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// 配置文件路径
    #[arg(short, long, default_value = "config.json")]
    config: PathBuf,
}

#[derive(Subcommand)]
enum Commands {
    /// 启动智能监控模式
    Monitor {
        /// 输出目录
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// 进程黑名单(逗号分隔)
        #[arg(short, long)]
        blacklist: Option<String>,
    },

    /// 测试麦克风检测
    TestDetection,

    /// 启动GUI界面
    Gui,
}

fn main() -> Result<()> {
    // 初始化日志
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Monitor { output, blacklist }) => {
            let mut config = RecorderConfig::default();

            if let Some(dir) = output {
                config.output_dir = dir;
            }

            if let Some(bl) = blacklist {
                config.blacklist = bl
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
            }

            let mut recorder = SmartRecorder::new(config);
            recorder.start_monitoring()?;
        }

        Some(Commands::TestDetection) => {
            test_detection()?;
        }

        Some(Commands::Gui) | None => {
            // 默认启动GUI
            println!("请使用 GUI 版本: smart-recorder-gui.exe");
        }
    }

    Ok(())
}

fn test_detection() -> Result<()> {
    use mic_detector::MicrophoneDetector;
    use std::time::Duration;

    println!("🎤 麦克风检测测试");
    println!("正在监控麦克风使用情况...\n");

    let mut detector = MicrophoneDetector::new();

    loop {
        match detector.detect_active_sessions() {
            Ok(sessions) => {
                if !sessions.is_empty() {
                    println!("检测到活跃会话:");
                    for session in sessions {
                        println!("  - {} (PID: {}, 类型: {})",
                            session.process_name,
                            session.process_id,
                            if session.is_capture { "麦克风" } else { "播放" }
                        );
                    }
                    println!();
                }
            }
            Err(e) => {
                eprintln!("检测错误: {}", e);
            }
        }

        std::thread::sleep(Duration::from_secs(2));
    }
}
