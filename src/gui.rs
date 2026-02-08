// GUI 版本仅支持 Windows
// GUI version only supports Windows

#[cfg(not(windows))]
fn main() {
    eprintln!("错误: GUI 版本仅支持 Windows 系统");
    eprintln!("Error: GUI version is only supported on Windows");
    eprintln!("");
    eprintln!("请使用命令行版本:");
    eprintln!("Please use the CLI version:");
    eprintln!("  auto-recorder --help");
    std::process::exit(1);
}

#[cfg(windows)]
mod gui_impl {
    use eframe::egui;
    use std::time::{Duration, Instant};

    // 录音模式
    #[derive(Debug, Clone, Copy, PartialEq)]
    enum RecordingMode {
        Manual,
        AutoOnAppStart,
        Scheduled,
    }

    // 应用状态
    pub struct RecorderApp {
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
        monitored_apps: String,
        
        save_in_mono: bool,
        output_path: String,
        create_monthly_folders: bool,
        create_daily_folders: bool,
        
        sample_rate: u32,
        bit_rate: u32,
        
        auto_save_duration: u32,
        silence_threshold: f32,
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
                
                monitored_apps: "QQ.exe,Skype.exe,WeChat.exe,Weixin.exe,WhatsApp.exe,WXWork.exe".to_string(),
                
                save_in_mono: true,
                output_path: "D:\\Documents\\录音".to_string(),
                create_monthly_folders: false,
                create_daily_folders: false,
                
                sample_rate: 48000,
                bit_rate: 128,
                
                auto_save_duration: 200,
                silence_threshold: 25.0,
            }
        }
    }

    impl RecorderApp {
        pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
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
        
        fn show_settings_ui(&mut self, ui: &mut egui::Ui) {
            ui.heading("录音模式");
            ui.add_space(10.0);
            
            ui.radio_value(&mut self.current_mode, RecordingMode::Manual, "手动录音");
            ui.add_space(5.0);
            
            ui.horizontal(|ui| {
                if ui.radio_value(&mut self.current_mode, RecordingMode::AutoOnAppStart, "软件启动后自动录音").changed() {
                    self.update_mode_text();
                }
                
                ui.add_space(20.0);
                ui.label("保存文件：");
                ui.add(egui::DragValue::new(&mut self.auto_save_duration).suffix(" 分钟"));
            });
            
            ui.add_space(5.0);
            
            ui.horizontal(|ui| {
                ui.checkbox(&mut true, "仅当该时间段内有音量超过后才指定值时才保存");
                ui.add(egui::DragValue::new(&mut self.silence_threshold).suffix(" %"));
            });
            
            ui.add_space(5.0);
            
            ui.horizontal(|ui| {
                if ui.radio(self.current_mode == RecordingMode::AutoOnAppStart, 
                           "当如下程序开始语音/视频聊天时自动开始录音").clicked() {
                    self.current_mode = RecordingMode::AutoOnAppStart;
                    self.update_mode_text();
                }
            });
            
            ui.add_space(5.0);
            
            ui.horizontal(|ui| {
                ui.add_space(25.0);
                ui.text_edit_singleline(&mut self.monitored_apps);
            });
            
            ui.add_space(10.0);
            
            ui.radio_value(&mut self.current_mode, RecordingMode::Scheduled, "计划录音模式");
            
            ui.add_space(15.0);
            ui.separator();
            ui.add_space(15.0);
            
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
            
            ui.horizontal(|ui| {
                ui.checkbox(&mut self.create_monthly_folders, "创建月份文件夹");
                ui.add_space(30.0);
                ui.checkbox(&mut self.create_daily_folders, "创建日期文件夹");
            });
            
            ui.add_space(15.0);
            
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

    impl eframe::App for RecorderApp {
        fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
            if self.is_recording && !self.is_paused {
                if let Some(start) = self.start_time {
                    self.recording_time = start.elapsed();
                }
                self.mic_level = (self.recording_time.as_secs_f32() * 2.0).sin().abs() * 0.8 + 0.2;
                self.speaker_level = (self.recording_time.as_secs_f32() * 1.5 + 1.0).sin().abs() * 0.7 + 0.3;
            }
            
            egui::CentralPanel::default().show(ctx, |ui| {
                ui.add_space(10.0);
                
                ui.horizontal(|ui| {
                    ui.add_space(10.0);
                    
                    ui.label(
                        egui::RichText::new(self.format_time())
                            .size(32.0)
                            .monospace()
                    );
                    
                    ui.add_space(20.0);
                    
                    let start_btn = egui::Button::new(
                        egui::RichText::new("开始").size(16.0)
                    ).min_size(egui::vec2(80.0, 35.0));
                    
                    if ui.add_enabled(!self.is_recording, start_btn).clicked() {
                        self.start_recording();
                    }
                    
                    ui.add_space(10.0);
                    
                    let stop_btn = egui::Button::new(
                        egui::RichText::new("停止").size(16.0)
                    ).min_size(egui::vec2(80.0, 35.0));
                    
                    if ui.add_enabled(self.is_recording, stop_btn).clicked() {
                        self.stop_recording();
                    }
                    
                    ui.add_space(20.0);
                    
                    let view_btn = egui::Button::new(
                        egui::RichText::new("查看").size(16.0)
                    ).min_size(egui::vec2(80.0, 35.0));
                    
                    if ui.add(view_btn).clicked() {
                        let _ = std::process::Command::new("explorer")
                            .arg(&self.output_path)
                            .spawn();
                    }
                });
                
                ui.add_space(15.0);
                
                ui.horizontal(|ui| {
                    ui.add_space(10.0);
                    ui.label(
                        egui::RichText::new(&self.mode_text)
                            .size(14.0)
                            .color(egui::Color32::from_rgb(100, 100, 100))
                    );
                });
                
                ui.add_space(10.0);
                
                ui.horizontal(|ui| {
                    ui.add_space(10.0);
                    ui.label(egui::RichText::new("🎤").size(20.0));
                    ui.add_space(5.0);
                    let mic_bar = egui::ProgressBar::new(self.mic_level)
                        .desired_width(ui.available_width() - 20.0);
                    ui.add(mic_bar);
                });
                
                ui.add_space(5.0);
                
                ui.horizontal(|ui| {
                    ui.add_space(10.0);
                    ui.label(egui::RichText::new("🔊").size(20.0));
                    ui.add_space(5.0);
                    let speaker_bar = egui::ProgressBar::new(self.speaker_level)
                        .desired_width(ui.available_width() - 20.0);
                    ui.add(speaker_bar);
                });
                
                ui.add_space(20.0);
                
                ui.horizontal(|ui| {
                    ui.add_space(10.0);
                    
                    if ui.button(egui::RichText::new("⚙ 设置").size(14.0)).clicked() {
                        self.show_settings = true;
                    }
                });
            });
            
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
            
            ctx.request_repaint();
        }
    }

    fn configure_fonts(ctx: &egui::Context) {
        let mut fonts = egui::FontDefinitions::default();
        
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
        
        ctx.set_fonts(fonts);
    }

    pub fn run_app() -> Result<(), eframe::Error> {
        let options = eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_inner_size([700.0, 350.0])
                .with_resizable(true),
            ..Default::default()
        };
        
        eframe::run_native(
            "Auto Recorder - 自动录音",
            options,
            Box::new(|cc| Box::new(RecorderApp::new(cc))),
        )
    }
}

#[cfg(windows)]
fn main() -> Result<(), eframe::Error> {
    gui_impl::run_app()
}
