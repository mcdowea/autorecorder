#[cfg(feature = "gui")]
use crate::config::RecorderConfig;
use crate::recorder::{RecorderManager, RecordingState};
use eframe::egui;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct RecorderApp {
    config: RecorderConfig,
    recorder_manager: Arc<RwLock<Option<RecorderManager>>>,
    runtime: tokio::runtime::Runtime,
    status_message: String,
    auto_record_enabled: bool,
    is_recording: bool,
}

impl RecorderApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let config = RecorderConfig::load().unwrap_or_default();
        let auto_record_enabled = config.auto_record;
        
        Self {
            config,
            recorder_manager: Arc::new(RwLock::new(None)),
            runtime: tokio::runtime::Runtime::new().unwrap(),
            status_message: "就绪".to_string(),
            auto_record_enabled,
            is_recording: false,
        }
    }
    
    fn start_recorder(&mut self) {
        let config = self.config.clone();
        let recorder_manager = Arc::clone(&self.recorder_manager);
        
        self.runtime.spawn(async move {
            match RecorderManager::new(config) {
                Ok(mut manager) => {
                    if let Err(e) = manager.start().await {
                        eprintln!("启动录音器失败: {}", e);
                    } else {
                        *recorder_manager.write().await = Some(manager);
                    }
                }
                Err(e) => {
                    eprintln!("创建录音器失败: {}", e);
                }
            }
        });
        
        self.status_message = "录音器已启动".to_string();
    }
    
    fn stop_recorder(&mut self) {
        let recorder_manager = Arc::clone(&self.recorder_manager);
        
        self.runtime.spawn(async move {
            if let Some(mut manager) = recorder_manager.write().await.take() {
                if let Err(e) = manager.stop().await {
                    eprintln!("停止录音器失败: {}", e);
                }
            }
        });
        
        self.status_message = "录音器已停止".to_string();
    }
    
    fn start_recording(&mut self) {
        let recorder_manager = Arc::clone(&self.recorder_manager);
        
        self.runtime.spawn(async move {
            if let Some(manager) = recorder_manager.read().await.as_ref() {
                if let Err(e) = manager.start_recording().await {
                    eprintln!("开始录音失败: {}", e);
                }
            }
        });
        
        self.is_recording = true;
        self.status_message = "正在录音...".to_string();
    }
    
    fn stop_recording(&mut self) {
        let recorder_manager = Arc::clone(&self.recorder_manager);
        let mut status_msg = self.status_message.clone();
        
        self.runtime.spawn(async move {
            if let Some(manager) = recorder_manager.read().await.as_ref() {
                match manager.stop_recording().await {
                    Ok(Some(path)) => {
                        println!("录音已保存: {:?}", path);
                    }
                    Ok(None) => {
                        println!("录音未保存（时长不足）");
                    }
                    Err(e) => {
                        eprintln!("停止录音失败: {}", e);
                    }
                }
            }
        });
        
        self.is_recording = false;
        self.status_message = "录音已停止".to_string();
    }
}

impl eframe::App for RecorderApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("自动录音程序");
            ui.separator();
            
            // 状态显示
            ui.horizontal(|ui| {
                ui.label("状态:");
                ui.colored_label(
                    if self.is_recording {
                        egui::Color32::RED
                    } else {
                        egui::Color32::GREEN
                    },
                    &self.status_message,
                );
            });
            
            ui.separator();
            
            // 自动录音设置
            ui.horizontal(|ui| {
                if ui.checkbox(&mut self.auto_record_enabled, "启用自动录音").changed() {
                    self.config.auto_record = self.auto_record_enabled;
                    let _ = self.config.save();
                }
            });
            
            ui.separator();
            
            // 手动控制
            ui.heading("手动控制");
            
            ui.horizontal(|ui| {
                if ui.button("▶ 启动录音器").clicked() {
                    self.start_recorder();
                }
                
                if ui.button("⏹ 停止录音器").clicked() {
                    self.stop_recorder();
                }
            });
            
            ui.horizontal(|ui| {
                if ui.button("🔴 开始录音").clicked() {
                    self.start_recording();
                }
                
                if ui.button("⏸ 停止录音").clicked() {
                    self.stop_recording();
                }
            });
            
            ui.separator();
            
            // 音频设置
            ui.heading("音频设置");
            
            ui.horizontal(|ui| {
                ui.label("采样率:");
                ui.add(egui::DragValue::new(&mut self.config.audio.sample_rate)
                    .speed(100)
                    .clamp_range(8000..=48000));
                ui.label("Hz");
            });
            
            ui.horizontal(|ui| {
                ui.label("比特率:");
                ui.add(egui::DragValue::new(&mut self.config.audio.bitrate)
                    .speed(8)
                    .clamp_range(64..=320));
                ui.label("kbps");
            });
            
            ui.horizontal(|ui| {
                ui.label("质量:");
                ui.add(egui::Slider::new(&mut self.config.audio.quality, 0..=9)
                    .text("(0=最高)"));
            });
            
            if ui.button("💾 保存设置").clicked() {
                if let Err(e) = self.config.save() {
                    self.status_message = format!("保存设置失败: {}", e);
                } else {
                    self.status_message = "设置已保存".to_string();
                }
            }
            
            ui.separator();
            
            // 输出目录
            ui.horizontal(|ui| {
                ui.label("输出目录:");
                ui.label(self.config.output_dir.display().to_string());
            });
            
            if ui.button("📁 打开输出目录").clicked() {
                #[cfg(target_os = "windows")]
                {
                    let _ = std::process::Command::new("explorer")
                        .arg(&self.config.output_dir)
                        .spawn();
                }
                
                #[cfg(target_os = "macos")]
                {
                    let _ = std::process::Command::new("open")
                        .arg(&self.config.output_dir)
                        .spawn();
                }
                
                #[cfg(target_os = "linux")]
                {
                    let _ = std::process::Command::new("xdg-open")
                        .arg(&self.config.output_dir)
                        .spawn();
                }
            }
        });
        
        // 定期刷新界面
        ctx.request_repaint_after(std::time::Duration::from_millis(500));
    }
}

#[cfg(feature = "gui")]
pub fn run_gui() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([500.0, 600.0])
            .with_resizable(true),
        ..Default::default()
    };
    
    eframe::run_native(
        "自动录音程序",
        options,
        Box::new(|cc| Ok(Box::new(RecorderApp::new(cc)))),
    )
}

#[cfg(not(feature = "gui"))]
pub fn run_gui() -> Result<(), Box<dyn std::error::Error>> {
    Err("GUI feature is not enabled. Compile with --features gui".into())
}
