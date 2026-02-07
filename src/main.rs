mod audio_capture;
mod config;
mod encoder;
mod gui;
mod process_monitor;
mod recorder;

use anyhow::Result;
use clap::{Parser, Subcommand};
use config::RecorderConfig;
use process_monitor::monitor_processes;
use recorder::RecorderManager;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, Level};
use tracing_subscriber;

#[derive(Parser)]
#[command(name = "auto-audio-recorder")]
#[command(about = "自动录音程序 - 支持自动检测通话并录音", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// 运行 GUI 界面
    Gui,
    /// 后台运行自动录音
    Run {
        /// 禁用自动录音
        #[arg(long)]
        no_auto: bool,
    },
    /// 手动开始录音
    Start,
    /// 显示配置
    Config,
    /// 列出音频设备
    Devices,
}

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .with_target(false)
        .init();
    
    let cli = Cli::parse();
    
    match cli.command {
        Some(Commands::Gui) => {
            info!("启动 GUI 界面...");
            #[cfg(feature = "gui")]
            {
                gui::run_gui()?;
            }
            #[cfg(not(feature = "gui"))]
            {
                eprintln!("GUI 功能未启用。请使用 --features gui 重新编译");
                eprintln!("或使用命令行模式: auto-audio-recorder run");
            }
        }
        Some(Commands::Run { no_auto }) => {
            run_auto_recorder(!no_auto).await?;
        }
        Some(Commands::Start) => {
            manual_record().await?;
        }
        Some(Commands::Config) => {
            show_config()?;
        }
        Some(Commands::Devices) => {
            list_devices()?;
        }
        None => {
            // 默认启动 GUI（如果启用）
            #[cfg(feature = "gui")]
            {
                info!("启动 GUI 界面...");
                gui::run_gui()?;
            }
            #[cfg(not(feature = "gui"))]
            {
                println!("自动录音程序 - Windows 版本");
                println!();
                println!("使用方法:");
                println!("  auto-audio-recorder run      # 自动录音模式");
                println!("  auto-audio-recorder start    # 手动录音");
                println!("  auto-audio-recorder config   # 显示配置");
                println!("  auto-audio-recorder devices  # 列出设备");
                println!("  auto-audio-recorder --help   # 显示帮助");
            }
        }
    }
    
    Ok(())
}

/// 运行自动录音模式
async fn run_auto_recorder(auto_enabled: bool) -> Result<()> {
    info!("启动自动录音模式...");
    
    let mut config = RecorderConfig::load()?;
    config.auto_record = auto_enabled;
    config.ensure_output_dir()?;
    
    info!("配置已加载:");
    info!("  输出目录: {:?}", config.output_dir);
    info!("  采样率: {} Hz", config.audio.sample_rate);
    info!("  比特率: {} kbps", config.audio.bitrate);
    info!("  自动录音: {}", config.auto_record);
    
    // 创建录音管理器
    let mut recorder_manager = RecorderManager::new(config.clone())?;
    recorder_manager.start().await?;
    
    info!("录音器已启动");
    
    if config.auto_record {
        // 启动进程监控
        let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);
        let mut app_status_rx = monitor_processes(config.monitored_apps.clone(), shutdown_rx).await;
        
        let mut is_recording = false;
        
        info!("进程监控已启动，监控应用: {:?}", config.monitored_apps);
        
        // 监听进程状态变化
        loop {
            tokio::select! {
                Ok(_) = app_status_rx.changed() => {
                    let apps_running = *app_status_rx.borrow();
                    
                    if apps_running && !is_recording {
                        info!("检测到通话应用运行，开始录音");
                        recorder_manager.start_recording().await?;
                        is_recording = true;
                    } else if !apps_running && is_recording {
                        info!("通话应用已关闭，停止录音");
                        if let Some(path) = recorder_manager.stop_recording().await? {
                            info!("录音已保存到: {:?}", path);
                        }
                        is_recording = false;
                    }
                }
                _ = tokio::signal::ctrl_c() => {
                    info!("收到退出信号");
                    break;
                }
            }
        }
        
        // 停止录音（如果正在录音）
        if is_recording {
            info!("停止录音...");
            if let Some(path) = recorder_manager.stop_recording().await? {
                info!("录音已保存到: {:?}", path);
            }
        }
        
        // 关闭进程监控
        let _ = shutdown_tx.send(true);
    } else {
        info!("自动录音已禁用，等待手动控制...");
        
        // 等待 Ctrl+C
        tokio::signal::ctrl_c().await?;
    }
    
    // 停止录音器
    recorder_manager.stop().await?;
    info!("录音器已停止");
    
    Ok(())
}

/// 手动录音
async fn manual_record() -> Result<()> {
    info!("开始手动录音...");
    
    let config = RecorderConfig::load()?;
    let mut recorder_manager = RecorderManager::new(config)?;
    
    recorder_manager.start().await?;
    recorder_manager.start_recording().await?;
    
    info!("正在录音... 按 Ctrl+C 停止");
    
    tokio::signal::ctrl_c().await?;
    
    info!("停止录音...");
    if let Some(path) = recorder_manager.stop_recording().await? {
        info!("录音已保存到: {:?}", path);
    }
    
    recorder_manager.stop().await?;
    
    Ok(())
}

/// 显示配置
fn show_config() -> Result<()> {
    let config = RecorderConfig::load()?;
    
    println!("当前配置:");
    println!("  输出目录: {:?}", config.output_dir);
    println!("  采样率: {} Hz", config.audio.sample_rate);
    println!("  比特率: {} kbps", config.audio.bitrate);
    println!("  声道数: {}", config.audio.channels);
    println!("  质量: {} (0-9, 0最高)", config.audio.quality);
    println!("  自动录音: {}", config.auto_record);
    println!("  最小通话时长: {} 秒", config.min_call_duration);
    println!("  监控应用:");
    for app in &config.monitored_apps {
        println!("    - {}", app);
    }
    
    Ok(())
}

/// 列出音频设备
fn list_devices() -> Result<()> {
    use audio_capture::AudioCapture;
    
    println!("可用音频设备:");
    
    let capture = AudioCapture::new()?;
    let devices = capture.list_devices()?;
    
    for device in devices {
        println!("  {}", device);
    }
    
    Ok(())
}
