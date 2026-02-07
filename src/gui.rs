mod audio_capture;
mod config;
mod mp3_encoder;
mod process_monitor;
mod recorder_core;

use eframe::egui;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use chrono::Local;

// 录音模式
#[derive(Debug, Clone, Copy, PartialEq)]
enum RecordingMode {
    Manual,
    AutoOnAppStart,
    Scheduled,
}

// 应用状态
struct RecorderApp {
    // 计时器
    recording_time: Duration,
    start_time: Option<Instant>,
    is_recording: bool,
    is_paused: bool,
    
    // 当前模式
    current_mode: RecordingMode,
    mode_text: String,
    
    // 音频电平
    mic_level: f32,
    speaker_level: f32,
    
    // 设置窗口
    show_settings: bool,
    
    // 配置
    manual_mode: bool,
    auto_mode: bool,
    scheduled_mode: bool,
    
    auto_save_duration: u32,  // 分钟
    silence_threshold: f32,   // 百分比
    
    monitored_apps: String,
    
    save_in_mono: bool,
    output_path: String,
    create_monthly_folders: bool,
    create_daily_folders: bool,
    
    sample_rate: u32,
    bit_rate: u32,
}

impl Default for RecorderApp {
    fn default() -> Self {
        Self {
            recording_time: Duration::from_secs(0),
            start_time: None,
            is_recording: false,
            is_paused: false,
            
            current_mode: RecordingMode::AutoOnAppStart,
            mode_text: "当前模式：语音视频聊天自动录音。".to_string(),
            
            mic_level: 0.0,
            speaker_level: 0.0,
            
            show_settings: false,
            
            manual_mode: false,
            auto_mode: true,
            scheduled_mode: false,
            
            auto_save_duration: 200,
            silence_threshold: 25.0,
            
            monitored_apps: "QQ.exe,Skype.exe,WeChat.exe,Weixin.exe,WhatsApp.exe,WXWork.exe".to_string(),
            
            save_in_mono: true,
            output_path: "D:\\Documents\\录音".to_string(),
            create_monthly_folders: false,
            create_daily_folders: false,
            
            sample_rate: 48000,
            bit_rate: 128,
        }
    }
}

impl RecorderApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // 设置中文字体
        configure_fonts(&cc.egui_ctx);
        Self::default()
    }
    
    fn update_mode_text(&mut self) {
        self.mode_text = match self.current_mode {
            RecordingMode::Manual => "当前模式：手动录音。".to_string(),
            RecordingMode::AutoOnAppStart => "当前模式：语音视频聊天自动录音。".to_string(),
            RecordingMode::Scheduled => "当前模式：计划录音。".to_string(),
        };
    }
    
    fn start_recording(&mut self) {
        self.is_recording = true;
        self.is_paused = false;
        self.start_time = Some(Instant::now());
    }
    
    fn pause_recording(&mut self) {
        self.is_paused = !self.is_paused;
    }
    
    fn stop_recording(&mut self) {
        self.is_recording = false;
        self.is_paused = false;
        self.recording_time = Duration::from_secs(0);
        self.start_time = None;
    }
    
    fn format_time(&self) -> String {
        let total_secs = self.recording_time.as_secs();
        let hours = total_secs / 3600;
        let minutes = (total_secs % 3600) / 60;
        let seconds = total_secs % 60;
        let millis = self.recording_time.subsec_millis() / 100;
        format!("{:02}:{:02}:{:02}.{}", hours, minutes, seconds, millis)
    }
}

impl eframe::App for RecorderApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // 更新录音时间
        if self.is_recording && !self.is_paused {
            if let Some(start) = self.start_time {
                self.recording_time = start.elapsed();
            }
            // 模拟音频电平变化
            self.mic_level = (self.recording_time.as_secs_f32() * 2.0).sin().abs() * 0.8 + 0.2;
            self.speaker_level = (self.recording_time.as_secs_f32() * 1.5 + 1.0).sin().abs() * 0.7 + 0.3;
        }
        
        // 主窗口
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add_space(10.0);
            
            // 顶部控制栏
            ui.horizontal(|ui| {
                ui.add_space(10.0);
                
                // 时间显示
                ui.label(
                    egui::RichText::new(self.format_time())
                        .size(32.0)
                        .monospace()
                );
                
                ui.add_space(20.0);
                
                // 开始按钮
                let start_btn = egui::Button::new(
                    egui::RichText::new("开始").size(16.0)
                ).min_size(egui::vec2(80.0, 35.0));
                
                if ui.add_enabled(!self.is_recording, start_btn).clicked() {
                    self.start_recording();
                }
                
                ui.add_space(10.0);
                
                // 停止按钮
                let stop_btn = egui::Button::new(
                    egui::RichText::new("停止").size(16.0)
                ).min_size(egui::vec2(80.0, 35.0));
                
                if ui.add_enabled(self.is_recording, stop_btn).clicked() {
                    self.stop_recording();
                }
                
                ui.add_space(20.0);
                
                // 查看按钮
                let view_btn = egui::Button::new(
                    egui::RichText::new("查看").size(16.0)
                ).min_size(egui::vec2(80.0, 35.0));
                
                if ui.add(view_btn).clicked() {
                    // 打开录音文件夹
                    #[cfg(target_os = "windows")]
                    {
                        let _ = std::process::Command::new("explorer")
                            .arg(&self.output_path)
                            .spawn();
                    }
                }
            });
            
            ui.add_space(15.0);
            
            // 模式提示
            ui.horizontal(|ui| {
                ui.add_space(10.0);
                ui.label(
                    egui::RichText::new(&self.mode_text)
                        .size(14.0)
                        .color(egui::Color32::from_rgb(100, 100, 100))
                );
            });
            
            ui.add_space(10.0);
            
            // 音频电平指示器
            ui.horizontal(|ui| {
                ui.add_space(10.0);
                
                // 麦克风图标
                ui.label(egui::RichText::new("🎤").size(20.0));
                ui.add_space(5.0);
                
                // 麦克风电平条
                let mic_bar = egui::ProgressBar::new(self.mic_level)
                    .desired_width(ui.available_width() - 20.0);
                ui.add(mic_bar);
            });
            
            ui.add_space(5.0);
            
            ui.horizontal(|ui| {
                ui.add_space(10.0);
                
                // 扬声器图标
                ui.label(egui::RichText::new("🔊").size(20.0));
                ui.add_space(5.0);
                
                // 扬声器电平条
                let speaker_bar = egui::ProgressBar::new(self.speaker_level)
                    .desired_width(ui.available_width() - 20.0);
                ui.add(speaker_bar);
            });
            
            ui.add_space(20.0);
            
            // 设置按钮
            ui.horizontal(|ui| {
                ui.add_space(10.0);
                
                if ui.button(egui::RichText::new("⚙ 设置").size(14.0)).clicked() {
                    self.show_settings = true;
                }
            });
        });
        
        // 设置窗口
        if self.show_settings {
            egui::Window::new("开机自动启动本软件")
                .collapsible(false)
                .resizable(true)
                .default_width(600.0)
                .show(ctx, |ui| {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        self.show_settings_ui(ui);
                    });
                    
                    ui.separator();
                    
                    ui.horizontal(|ui| {
                        if ui.button("关闭").clicked() {
                            self.show_settings = false;
                        }
                    });
                });
        }
        
        // 请求持续刷新以更新计时器
        ctx.request_repaint();
    }
}

impl RecorderApp {
    fn show_settings_ui(&mut self, ui: &mut egui::Ui) {
        ui.heading("录音模式");
        ui.add_space(10.0);
        
        // 手动录音
        ui.radio_value(&mut self.current_mode, RecordingMode::Manual, "手动录音");
        ui.add_space(5.0);
        
        // 软件启动后自动录音
        ui.horizontal(|ui| {
            if ui.radio_value(&mut self.current_mode, RecordingMode::AutoOnAppStart, "软件启动后自动录音").changed() {
                self.update_mode_text();
            }
            
            ui.add_space(20.0);
            ui.label("保存文件：");
            ui.add(egui::DragValue::new(&mut self.auto_save_duration).suffix(" 分钟"));
        });
        
        ui.add_space(5.0);
        
        // 静音阈值
        ui.horizontal(|ui| {
            ui.checkbox(&mut true, "仅当该时间段内有音量超过后才指定值时才保存");
            ui.add(egui::DragValue::new(&mut self.silence_threshold).suffix(" %"));
        });
        
        ui.add_space(5.0);
        
        // 当如下程序开始语音/视频聊天时自动开始录音
        ui.horizontal(|ui| {
            if ui.radio(self.current_mode == RecordingMode::AutoOnAppStart, 
                       "当如下程序开始语音/视频聊天时自动开始录音").clicked() {
                self.current_mode = RecordingMode::AutoOnAppStart;
                self.update_mode_text();
            }
        });
        
        ui.add_space(5.0);
        
        // 监控的应用程序
        ui.horizontal(|ui| {
            ui.add_space(25.0);
            ui.text_edit_singleline(&mut self.monitored_apps);
        });
        
        ui.add_space(10.0);
        
        // 计划录音模式
        ui.radio_value(&mut self.current_mode, RecordingMode::Scheduled, "计划录音模式");
        ui.add_space(5.0);
        
        // 计划表格
        ui.horizontal(|ui| {
            ui.add_space(25.0);
            
            egui::Grid::new("schedule_grid")
                .num_columns(3)
                .spacing([10.0, 5.0])
                .show(ui, |ui| {
                    ui.label("开始时间");
                    ui.label("录音时长");
                    ui.label("类型");
                    ui.end_row();
                });
            
            ui.add_space(20.0);
            ui.button("添加");
        });
        
        ui.add_space(5.0);
        
        ui.horizontal(|ui| {
            ui.add_space(25.0);
            ui.button("删除");
        });
        
        ui.add_space(10.0);
        
        // 手动开始录制
        ui.radio(false, "手动开始录制，并在设定时间后自动停止录制");
        
        ui.add_space(15.0);
        ui.separator();
        ui.add_space(15.0);
        
        // 音频源选择
        ui.heading("音频源");
        ui.add_space(10.0);
        
        ui.radio(false, "录制从麦克风输入的声音");
        ui.add_space(5.0);
        ui.radio(false, "录制从电脑播放的声音");
        ui.add_space(5.0);
        
        ui.horizontal(|ui| {
            ui.radio_value(&mut self.save_in_mono, true, "输入及播放的声音均进行录制");
            ui.add_space(20.0);
            ui.checkbox(&mut false, "声音保存在不同声道中");
        });
        
        ui.add_space(15.0);
        
        // 保存路径
        ui.horizontal(|ui| {
            ui.label("保存路径：");
            ui.text_edit_singleline(&mut self.output_path);
            if ui.button("浏览").clicked() {
                if let Some(path) = rfd::FileDialog::new().pick_folder() {
                    self.output_path = path.display().to_string();
                }
            }
        });
        
        ui.add_space(10.0);
        
        // 文件夹选项
        ui.horizontal(|ui| {
            ui.checkbox(&mut self.create_monthly_folders, "创建月份文件夹");
            ui.add_space(30.0);
            ui.checkbox(&mut self.create_daily_folders, "创建日期文件夹");
        });
        
        ui.add_space(15.0);
        
        // 采样频率和比特率
        ui.horizontal(|ui| {
            ui.label("采样频率：");
            egui::ComboBox::from_id_source("sample_rate")
                .selected_text(format!("{}", self.sample_rate))
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.sample_rate, 8000, "8000");
                    ui.selectable_value(&mut self.sample_rate, 11025, "11025");
                    ui.selectable_value(&mut self.sample_rate, 22050, "22050");
                    ui.selectable_value(&mut self.sample_rate, 44100, "44100");
                    ui.selectable_value(&mut self.sample_rate, 48000, "48000");
                });
            
            ui.add_space(30.0);
            
            ui.label("比特率：");
            egui::ComboBox::from_id_source("bit_rate")
                .selected_text(format!("{}", self.bit_rate))
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.bit_rate, 32, "32");
                    ui.selectable_value(&mut self.bit_rate, 64, "64");
                    ui.selectable_value(&mut self.bit_rate, 96, "96");
                    ui.selectable_value(&mut self.bit_rate, 128, "128");
                    ui.selectable_value(&mut self.bit_rate, 192, "192");
                    ui.selectable_value(&mut self.bit_rate, 256, "256");
                    ui.selectable_value(&mut self.bit_rate, 320, "320");
                });
        });
    }
}

fn configure_fonts(ctx: &egui::Context) {
    // 使用 egui 默认字体，它已经包含了基本的中文支持
    // 如果需要更好的中文支持，可以在 Windows 上使用微软雅黑等系统字体
    let mut fonts = egui::FontDefinitions::default();
    
    #[cfg(target_os = "windows")]
    {
        // Windows 系统尝试使用微软雅黑
        if let Ok(font_data) = std::fs::read("C:\\Windows\\Fonts\\msyh.ttc") {
            fonts.font_data.insert(
                "chinese".to_owned(),
                egui::FontData::from_owned(font_data),
            );
            
            fonts
                .families
                .entry(egui::FontFamily::Proportional)
                .or_default()
                .insert(0, "chinese".to_owned());
        }
    }
    
    ctx.set_fonts(fonts);
}

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(700.0, 350.0)),
        resizable: true,
        ..Default::default()
    };
    
    eframe::run_native(
        "Auto Recorder - 自动录音",
        options,
        Box::new(|cc| Box::new(RecorderApp::new(cc))),
    )
}
