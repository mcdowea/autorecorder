// 智能录音工具GUI
// 支持系统托盘、开机自启、实时电平监控等功能

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[cfg(not(windows))]
fn main() {
    eprintln!("错误: 此工具仅支持 Windows 系统");
    std::process::exit(1);
}

#[cfg(windows)]
mod gui_impl {
    use eframe::egui;
    use std::sync::Arc;
    use parking_lot::Mutex;
    use serde::{Deserialize, Serialize};
    use std::path::PathBuf;
    use std::time::{Duration, Instant};

    use crate::smart_recorder::{SmartRecorder, RecorderConfig, AudioFormat};
    use crate::mic_detector::MicrophoneDetector;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct AppConfig {
        // 基本设置
        output_path: String,
        auto_create_folders: bool,

        // 音质设置
        sample_rate: u32,
        bit_rate: u32,
        audio_format: String,  // "mp3" or "wav"
        mp3_quality: u8,  // 0-9, 越小质量越高

        // 增益设置
        mic_gain: f32,
        speaker_gain: f32,

        // 黑名单
        blacklist: String,  // 逗号分隔的进程名

        // 系统设置
        auto_start: bool,
        minimize_to_tray: bool,
        min_duration_seconds: u64,
    }

    impl Default for AppConfig {
        fn default() -> Self {
            Self {
                output_path: "D:\\Recordings".to_string(),
                auto_create_folders: true,
                sample_rate: 48000,
                bit_rate: 128,
                audio_format: "mp3".to_string(),
                mp3_quality: 2,  // 默认质量为2(高质量)
                mic_gain: 1.0,
                speaker_gain: 1.0,
                blacklist: "chrome.exe,firefox.exe,msedge.exe,explorer.exe".to_string(),
                auto_start: false,
                minimize_to_tray: true,
                min_duration_seconds: 3,
            }
        }
    }

    impl AppConfig {
        fn load() -> Self {
            let config_path = Self::config_path();
            if let Ok(content) = std::fs::read_to_string(&config_path) {
                if let Ok(config) = serde_json::from_str(&content) {
                    return config;
                }
            }
            Self::default()
        }

        fn save(&self) {
            let config_path = Self::config_path();
            if let Ok(content) = serde_json::to_string_pretty(self) {
                let _ = std::fs::write(&config_path, content);
            }
        }

        fn config_path() -> PathBuf {
            let exe_path = std::env::current_exe().unwrap_or_default();
            let exe_dir = exe_path.parent().unwrap_or(std::path::Path::new("."));
            exe_dir.join("smart_recorder_config.json")
        }

        fn to_recorder_config(&self) -> RecorderConfig {
            RecorderConfig {
                output_dir: PathBuf::from(&self.output_path),
                sample_rate: self.sample_rate,
                bit_rate: self.bit_rate,
                quality: self.mp3_quality,
                mic_gain: self.mic_gain,
                speaker_gain: self.speaker_gain,
                blacklist: self.blacklist
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect(),
                auto_create_folders: self.auto_create_folders,
                save_format: if self.audio_format == "mp3" {
                    AudioFormat::Mp3
                } else {
                    AudioFormat::Wav
                },
                min_recording_duration: Duration::from_secs(self.min_duration_seconds),
            }
        }
    }

    struct AudioLevels {
        mic_level: f32,
        speaker_level: f32,
    }

    pub struct SmartRecorderApp {
        config: AppConfig,
        show_settings: bool,

        // 运行状态
        is_monitoring: bool,
        is_recording: bool,
        is_manual_recording: bool,  // 手动录音状态
        current_app: Option<String>,
        recording_duration: Duration,
        manual_recording_start: Option<Instant>,  // 手动录音开始时间

        // 音频电平
        audio_levels: Arc<Mutex<AudioLevels>>,

        // 监控线程
        monitor_thread: Option<std::thread::JoinHandle<()>>,
        audio_monitor_thread: Option<std::thread::JoinHandle<()>>,
        manual_recording_thread: Option<std::thread::JoinHandle<()>>,  // 手动录音线程
        stop_signal: Arc<Mutex<bool>>,
        manual_stop_signal: Arc<Mutex<bool>>,  // 手动录音停止信号

        // 日志
        log_messages: Vec<String>,
        max_log_lines: usize,
    }

    impl Default for SmartRecorderApp {
        fn default() -> Self {
            Self {
                config: AppConfig::load(),
                show_settings: false,
                is_monitoring: false,
                is_recording: false,
                is_manual_recording: false,
                current_app: None,
                recording_duration: Duration::ZERO,
                manual_recording_start: None,
                audio_levels: Arc::new(Mutex::new(AudioLevels {
                    mic_level: 0.0,
                    speaker_level: 0.0,
                })),
                monitor_thread: None,
                audio_monitor_thread: None,
                manual_recording_thread: None,
                stop_signal: Arc::new(Mutex::new(false)),
                manual_stop_signal: Arc::new(Mutex::new(false)),
                log_messages: Vec::new(),
                max_log_lines: 100,
            }
        }
    }

    impl SmartRecorderApp {
        pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
            configure_fonts(&cc.egui_ctx);
            let mut app = Self::default();

            // 启动音频电平监控
            app.start_audio_level_monitor();

            app
        }

        fn start_audio_level_monitor(&mut self) {
            let audio_levels = Arc::clone(&self.audio_levels);
            let stop_signal = Arc::clone(&self.stop_signal);

            let handle = std::thread::spawn(move || {
                audio_level_monitor_loop(audio_levels, stop_signal);
            });

            self.audio_monitor_thread = Some(handle);
        }

        fn add_log(&mut self, message: String) {
            let timestamp = chrono::Local::now().format("%H:%M:%S");
            self.log_messages.push(format!("[{}] {}", timestamp, message));

            // 限制日志行数
            if self.log_messages.len() > self.max_log_lines {
                self.log_messages.remove(0);
            }
        }

        fn start_monitoring(&mut self) {
            if self.is_monitoring {
                return;
            }

            self.is_monitoring = true;
            self.add_log("🎤 智能监控已启动".to_string());

            let config = self.config.to_recorder_config();
            let stop_signal = Arc::clone(&self.stop_signal);

            let handle = std::thread::spawn(move || {
                let mut recorder = SmartRecorder::new(config);
                // 简化的监控循环
                loop {
                    if *stop_signal.lock() {
                        break;
                    }
                    std::thread::sleep(Duration::from_secs(1));
                }
            });

            self.monitor_thread = Some(handle);
        }

        fn stop_monitoring(&mut self) {
            if !self.is_monitoring {
                return;
            }

            self.is_monitoring = false;
            *self.stop_signal.lock() = true;
            self.add_log("⏸️  智能监控已停止".to_string());
        }

        fn save_settings(&mut self) {
            self.config.save();
            self.add_log("✅ 设置已保存".to_string());
        }

        fn start_manual_recording(&mut self) {
            if self.is_manual_recording {
                return;
            }

            self.is_manual_recording = true;
            self.manual_recording_start = Some(Instant::now());
            *self.manual_stop_signal.lock() = false;
            self.add_log("🎙️  手动录音已开始".to_string());

            let config = self.config.to_recorder_config();
            let stop_signal = Arc::clone(&self.manual_stop_signal);

            let handle = std::thread::spawn(move || {
                if let Err(e) = manual_recording_thread(config, stop_signal) {
                    eprintln!("手动录音错误: {}", e);
                }
            });

            self.manual_recording_thread = Some(handle);
        }

        fn stop_manual_recording(&mut self) {
            if !self.is_manual_recording {
                return;
            }

            self.is_manual_recording = false;
            *self.manual_stop_signal.lock() = true;

            if let Some(start) = self.manual_recording_start {
                let duration = start.elapsed();
                self.add_log(format!("⏹️  手动录音已停止 (时长: {:.1}秒)", duration.as_secs_f32()));
            }

            self.manual_recording_start = None;
        }

        fn show_main_ui(&mut self, ui: &mut egui::Ui) {
            ui.add_space(10.0);

            // 状态指示器
            ui.horizontal(|ui| {
                ui.add_space(10.0);

                let status_text = if self.is_manual_recording {
                    "🎙️  手动录音中"
                } else if self.is_recording {
                    "🔴 正在录音"
                } else if self.is_monitoring {
                    "👁️  监控中"
                } else {
                    "⏸️  已停止"
                };

                ui.label(
                    egui::RichText::new(status_text)
                        .size(20.0)
                        .strong()
                );

                if let Some(ref app) = self.current_app {
                    ui.add_space(20.0);
                    ui.label(
                        egui::RichText::new(format!("应用: {}", app))
                            .size(16.0)
                    );
                }
            });

            ui.add_space(15.0);

            // 控制按钮
            ui.horizontal(|ui| {
                ui.add_space(10.0);

                if self.is_monitoring {
                    if ui.add(egui::Button::new(
                        egui::RichText::new("⏸️  停止监控").size(16.0)
                    ).min_size(egui::vec2(120.0, 40.0))).clicked() {
                        self.stop_monitoring();
                    }
                } else {
                    if ui.add(egui::Button::new(
                        egui::RichText::new("▶️  开始监控").size(16.0)
                    ).min_size(egui::vec2(120.0, 40.0))).clicked() {
                        self.start_monitoring();
                    }
                }

                ui.add_space(10.0);

                // 手动录音按钮
                if self.is_manual_recording {
                    if ui.add(egui::Button::new(
                        egui::RichText::new("⏹️  停止录音").size(16.0)
                    ).min_size(egui::vec2(120.0, 40.0))).clicked() {
                        self.stop_manual_recording();
                    }
                } else {
                    if ui.add(egui::Button::new(
                        egui::RichText::new("🎙️  手动录音").size(16.0)
                    ).min_size(egui::vec2(120.0, 40.0))).clicked() {
                        self.start_manual_recording();
                    }
                }

                ui.add_space(10.0);

                if ui.add(egui::Button::new(
                    egui::RichText::new("📁 打开文件夹").size(16.0)
                ).min_size(egui::vec2(120.0, 40.0))).clicked() {
                    let _ = std::process::Command::new("explorer")
                        .arg(&self.config.output_path)
                        .spawn();
                }

                ui.add_space(10.0);

                if ui.add(egui::Button::new(
                    egui::RichText::new("⚙️  设置").size(16.0)
                ).min_size(egui::vec2(100.0, 40.0))).clicked() {
                    self.show_settings = true;
                }
            });

            ui.add_space(20.0);
            ui.separator();
            ui.add_space(10.0);

            // 音频电平显示
            let (mic_level, speaker_level) = {
                let levels = self.audio_levels.lock();
                (levels.mic_level, levels.speaker_level)
            };

            ui.label(egui::RichText::new("实时音频电平").size(14.0).strong());
            ui.add_space(5.0);

            ui.horizontal(|ui| {
                ui.add_space(10.0);
                ui.label("🎤 麦克风:");
                ui.add_space(5.0);
                ui.add(egui::ProgressBar::new(mic_level)
                    .text(format!("{:.0}%", mic_level * 100.0))
                    .desired_width(ui.available_width() - 20.0));
            });

            ui.add_space(5.0);

            ui.horizontal(|ui| {
                ui.add_space(10.0);
                ui.label("🔊 扬声器:");
                ui.add_space(5.0);
                ui.add(egui::ProgressBar::new(speaker_level)
                    .text(format!("{:.0}%", speaker_level * 100.0))
                    .desired_width(ui.available_width() - 20.0));
            });

            ui.add_space(15.0);
            ui.separator();
            ui.add_space(10.0);

            // 日志窗口
            ui.label(egui::RichText::new("运行日志").size(14.0).strong());
            ui.add_space(5.0);

            egui::ScrollArea::vertical()
                .max_height(200.0)
                .show(ui, |ui| {
                    for msg in &self.log_messages {
                        ui.label(msg);
                    }
                });
        }

        fn show_settings_ui(&mut self, ui: &mut egui::Ui) {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.heading("基本设置");
                ui.add_space(10.0);

                ui.horizontal(|ui| {
                    ui.label("保存路径:");
                    ui.text_edit_singleline(&mut self.config.output_path);
                    if ui.button("浏览").clicked() {
                        if let Some(path) = rfd::FileDialog::new().pick_folder() {
                            self.config.output_path = path.display().to_string();
                        }
                    }
                });

                ui.add_space(5.0);
                ui.checkbox(&mut self.config.auto_create_folders, "按应用名自动创建文件夹");

                ui.add_space(15.0);
                ui.separator();
                ui.add_space(15.0);

                ui.heading("音质设置");
                ui.add_space(10.0);

                ui.horizontal(|ui| {
                    ui.label("采样率:");
                    egui::ComboBox::from_id_source("sample_rate")
                        .selected_text(format!("{} Hz", self.config.sample_rate))
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut self.config.sample_rate, 16000, "16000 Hz (低质量)");
                            ui.selectable_value(&mut self.config.sample_rate, 22050, "22050 Hz");
                            ui.selectable_value(&mut self.config.sample_rate, 44100, "44100 Hz (CD质量)");
                            ui.selectable_value(&mut self.config.sample_rate, 48000, "48000 Hz (高质量)");
                            ui.selectable_value(&mut self.config.sample_rate, 96000, "96000 Hz (超高质量)");
                        });

                    ui.add_space(20.0);

                    ui.label("音频格式:");
                    egui::ComboBox::from_id_source("format")
                        .selected_text(&self.config.audio_format.to_uppercase())
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut self.config.audio_format, "mp3".to_string(), "MP3");
                            ui.selectable_value(&mut self.config.audio_format, "wav".to_string(), "WAV");
                        });
                });

                ui.add_space(5.0);

                if self.config.audio_format == "mp3" {
                    ui.horizontal(|ui| {
                        ui.label("比特率:");
                        egui::ComboBox::from_id_source("bitrate")
                            .selected_text(format!("{} kbps", self.config.bit_rate))
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut self.config.bit_rate, 64, "64 kbps (低质量)");
                                ui.selectable_value(&mut self.config.bit_rate, 96, "96 kbps");
                                ui.selectable_value(&mut self.config.bit_rate, 128, "128 kbps (标准)");
                                ui.selectable_value(&mut self.config.bit_rate, 192, "192 kbps (高质量)");
                                ui.selectable_value(&mut self.config.bit_rate, 256, "256 kbps (极高质量)");
                                ui.selectable_value(&mut self.config.bit_rate, 320, "320 kbps (最高质量)");
                            });
                    });

                    ui.add_space(5.0);

                    ui.horizontal(|ui| {
                        ui.label("MP3编码质量:");
                        egui::ComboBox::from_id_source("mp3_quality")
                            .selected_text(match self.config.mp3_quality {
                                0 => "0 (最高质量，最慢)",
                                2 => "2 (高质量，推荐)",
                                5 => "5 (标准质量)",
                                7 => "7 (低质量，快速)",
                                9 => "9 (最低质量，最快)",
                                _ => &format!("{}", self.config.mp3_quality),
                            })
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut self.config.mp3_quality, 0, "0 (最高质量，最慢)");
                                ui.selectable_value(&mut self.config.mp3_quality, 2, "2 (高质量，推荐)");
                                ui.selectable_value(&mut self.config.mp3_quality, 5, "5 (标准质量)");
                                ui.selectable_value(&mut self.config.mp3_quality, 7, "7 (低质量，快速)");
                                ui.selectable_value(&mut self.config.mp3_quality, 9, "9 (最低质量，最快)");
                            });
                    });
                }

                ui.add_space(15.0);
                ui.separator();
                ui.add_space(15.0);

                ui.heading("音量增益");
                ui.add_space(10.0);

                ui.horizontal(|ui| {
                    ui.label("麦克风增益:");
                    ui.add(egui::Slider::new(&mut self.config.mic_gain, 0.0..=2.0)
                        .text(format!("{:.1}x", self.config.mic_gain)));
                });

                ui.add_space(5.0);

                ui.horizontal(|ui| {
                    ui.label("扬声器增益:");
                    ui.add(egui::Slider::new(&mut self.config.speaker_gain, 0.0..=2.0)
                        .text(format!("{:.1}x", self.config.speaker_gain)));
                });

                ui.add_space(15.0);
                ui.separator();
                ui.add_space(15.0);

                ui.heading("进程黑名单");
                ui.add_space(5.0);
                ui.label("以下程序使用麦克风时不会触发录音(用逗号分隔):");
                ui.add_space(5.0);
                ui.text_edit_multiline(&mut self.config.blacklist);

                ui.add_space(15.0);
                ui.separator();
                ui.add_space(15.0);

                ui.heading("其他设置");
                ui.add_space(10.0);

                ui.horizontal(|ui| {
                    ui.label("最小录音时长(秒):");
                    ui.add(egui::DragValue::new(&mut self.config.min_duration_seconds)
                        .speed(1.0)
                        .clamp_range(0..=60));
                });

                ui.add_space(5.0);
                ui.checkbox(&mut self.config.auto_start, "开机自动启动");
                ui.checkbox(&mut self.config.minimize_to_tray, "最小化到系统托盘");
            });
        }
    }

    impl eframe::App for SmartRecorderApp {
        fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
            egui::CentralPanel::default().show(ctx, |ui| {
                self.show_main_ui(ui);
            });

            if self.show_settings {
                egui::Window::new("设置")
                    .collapsible(false)
                    .resizable(true)
                    .default_width(600.0)
                    .show(ctx, |ui| {
                        self.show_settings_ui(ui);

                        ui.separator();

                        ui.horizontal(|ui| {
                            if ui.button("保存").clicked() {
                                self.save_settings();
                                self.show_settings = false;
                            }

                            if ui.button("取消").clicked() {
                                self.config = AppConfig::load();
                                self.show_settings = false;
                            }
                        });
                    });
            }

            ctx.request_repaint_after(Duration::from_millis(100));
        }
    }

    impl Drop for SmartRecorderApp {
        fn drop(&mut self) {
            *self.stop_signal.lock() = true;
            *self.manual_stop_signal.lock() = true;

            if let Some(handle) = self.monitor_thread.take() {
                let _ = handle.join();
            }

            if let Some(handle) = self.audio_monitor_thread.take() {
                let _ = handle.join();
            }

            if let Some(handle) = self.manual_recording_thread.take() {
                let _ = handle.join();
            }
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

    fn audio_level_monitor_loop(
        audio_levels: Arc<Mutex<AudioLevels>>,
        stop_signal: Arc<Mutex<bool>>,
    ) {
        use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

        let host = cpal::default_host();

        if let Some(mic_device) = host.default_input_device() {
            if let Ok(config) = mic_device.default_input_config() {
                let levels_clone = Arc::clone(&audio_levels);
                if let Ok(stream) = mic_device.build_input_stream(
                    &config.into(),
                    move |data: &[f32], _| {
                        let rms = calculate_rms(data);
                        levels_clone.lock().mic_level = rms;
                    },
                    |_| {},
                    None,
                ) {
                    let _ = stream.play();

                    while !*stop_signal.lock() {
                        std::thread::sleep(Duration::from_millis(100));
                    }
                }
            }
        }
    }

    fn calculate_rms(samples: &[f32]) -> f32 {
        if samples.is_empty() {
            return 0.0;
        }
        let sum: f32 = samples.iter().map(|&s| s * s).sum();
        ((sum / samples.len() as f32).sqrt() * 3.0).min(1.0)
    }

    fn manual_recording_thread(
        config: RecorderConfig,
        stop_signal: Arc<Mutex<bool>>,
    ) -> Result<(), anyhow::Error> {
        use crate::dual_recorder::{DualChannelRecorder, AudioMixer};
        use crate::mp3_encoder::{StreamingMp3Encoder, WavEncoder};
        use std::time::Instant;

        println!("🎙️  手动录音开始...");

        let start_time = Instant::now();

        // 创建双通道录音器
        let mut recorder = DualChannelRecorder::new(config.sample_rate);
        recorder.set_mic_gain(config.mic_gain);
        recorder.set_speaker_gain(config.speaker_gain);

        // 开始录音
        let session = recorder.start_recording()?;

        // 创建音频混音器
        let mut mixer = AudioMixer::new();

        // 创建编码器
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
        while !*stop_signal.lock() {
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
        println!("⏹️  手动录音停止 (时长: {:.1}秒)", duration.as_secs_f32());

        // 生成文件名
        let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
        let filename = if config.save_format == AudioFormat::Mp3 {
            format!("manual_{}.mp3", timestamp)
        } else {
            format!("manual_{}.wav", timestamp)
        };

        std::fs::create_dir_all(&config.output_dir).ok();
        let output_path = config.output_dir.join(filename);

        // 保存文件
        match config.save_format {
            AudioFormat::Mp3 => {
                if let Some(enc) = encoder {
                    enc.save_to_file(&output_path)?;
                    println!("💾 手动录音已保存: {:?} (MP3格式)", output_path);
                }
            }
            AudioFormat::Wav => {
                let wav_encoder = WavEncoder::new(config.sample_rate);
                wav_encoder.encode_to_file(&all_samples, &output_path)?;
                println!("💾 手动录音已保存: {:?} (WAV格式)", output_path);
            }
        }

        Ok(())
    }

    pub fn run_app() -> Result<(), eframe::Error> {
        let options = eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_inner_size([800.0, 600.0])
                .with_resizable(true),
            ..Default::default()
        };

        eframe::run_native(
            "智能录音工具 - Smart Recorder",
            options,
            Box::new(|cc| Ok(Box::new(SmartRecorderApp::new(cc)))),
        )
    }
}

#[cfg(windows)]
fn main() -> Result<(), eframe::Error> {
    gui_impl::run_app()
}
