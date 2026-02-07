# 项目文件总览

## 📦 完整的自动录音程序

这是一个使用纯 Rust 实现的自动录音程序，无需 FFmpeg 和 lame_enc.dll 等外部依赖。

### 🎯 核心功能

✅ **同时录制麦克风和扬声器**  
✅ **自动检测通话软件**（微信、QQ、飞书、Skype 等）  
✅ **自动开始/停止录音**  
✅ **纯 Rust MP3 编码**  
✅ **可配置音质参数**（采样率、比特率、质量）  
✅ **图形界面 + 命令行**  
✅ **GitHub Actions 自动发布**  

---

## 📂 项目结构

```
audio_recorder/
├── src/
│   ├── main.rs              # 主程序入口，命令行处理
│   ├── config.rs            # 配置文件管理
│   ├── audio_capture.rs     # 音频捕获（麦克风+扬声器）
│   ├── encoder.rs           # MP3 编码器（纯 Rust）
│   ├── recorder.rs          # 录音器核心逻辑
│   ├── process_monitor.rs   # 进程监控（检测通话软件）
│   └── gui.rs              # GUI 图形界面
│
├── .github/
│   └── workflows/
│       └── release.yml      # GitHub Actions 自动构建和发布
│
├── Cargo.toml               # Rust 项目配置
├── config.example.toml      # 配置文件示例（含详细说明）
│
├── README.md                # 英文说明文档
├── README_ZH.md             # 中文说明文档
├── BUILD.md                 # 编译构建指南
├── QUICKSTART.md            # 快速使用指南
├── LICENSE                  # MIT 许可证
└── .gitignore              # Git 忽略文件
```

---

## 🚀 快速使用

### 1. 编译项目

```bash
# 进入项目目录
cd audio_recorder

# 编译发布版本
cargo build --release

# 可执行文件在 target/release/auto-audio-recorder
```

### 2. 运行程序

```bash
# GUI 模式（推荐新手）
./target/release/auto-audio-recorder gui

# 自动录音模式
./target/release/auto-audio-recorder run

# 手动录音
./target/release/auto-audio-recorder start

# 查看配置
./target/release/auto-audio-recorder config

# 列出设备
./target/release/auto-audio-recorder devices
```

### 3. 配置修改

配置文件会在首次运行时自动创建在：
- Windows: `%APPDATA%\auto-audio-recorder\config.toml`
- macOS: `~/Library/Application Support/auto-audio-recorder/config.toml`
- Linux: `~/.config/auto-audio-recorder/config.toml`

参考 `config.example.toml` 进行配置。

---

## 📋 关键文件说明

### 源代码文件

| 文件 | 功能说明 | 关键技术 |
|-----|---------|---------|
| `main.rs` | 程序入口，命令行参数处理 | clap, tokio |
| `config.rs` | 配置加载、保存、验证 | serde, toml |
| `audio_capture.rs` | 音频设备枚举和捕获 | cpal, Windows WASAPI |
| `encoder.rs` | PCM 转 MP3 编码 | mp3lame-encoder |
| `recorder.rs` | 录音控制、文件保存 | Arc, Mutex, 异步处理 |
| `process_monitor.rs` | 监控指定进程是否运行 | sysinfo |
| `gui.rs` | 图形用户界面 | egui, eframe |

### 配置和文档

| 文件 | 说明 |
|-----|------|
| `Cargo.toml` | Rust 项目依赖和元数据 |
| `config.example.toml` | 配置文件模板，含详细注释 |
| `README.md` | 英文项目说明 |
| `README_ZH.md` | 中文项目说明 |
| `BUILD.md` | 编译、构建、发布指南 |
| `QUICKSTART.md` | 5 分钟快速上手 |

### CI/CD

| 文件 | 说明 |
|-----|------|
| `.github/workflows/release.yml` | 自动构建 Windows/macOS/Linux 版本 |

---

## 🎨 技术亮点

### 1. 纯 Rust 实现
- ✅ 无需 FFmpeg
- ✅ 无需 lame_enc.dll
- ✅ 所有依赖都是 Rust crate
- ✅ 跨平台编译，无额外配置

### 2. 双通道同时录制
```rust
// 同时捕获麦克风和扬声器
let mic_stream = audio_capture.create_capture_stream(...);
let speaker_stream = windows_loopback::create_loopback_stream(...);

// 混音合并
let mixed = Mp3Encoder::mix_channels(&mic_data, &speaker_data);
```

### 3. 智能进程检测
```rust
// 监控通话软件
let apps = vec!["WeChat.exe", "QQ.exe", "Skype.exe"];
let monitor = ProcessMonitor::new(apps);

// 检测到应用运行时自动开始录音
if monitor.check_apps_running() {
    recorder.start_recording().await?;
}
```

### 4. 高质量 MP3 编码
```rust
// 可配置参数
let encoder = Mp3Encoder::new(
    sample_rate: 44100,
    channels: 2,
    bitrate: 128,
    quality: 2,  // 0-9, 0 最高质量
);
```

---

## ⚙️ 配置选项

### 基础配置
```toml
output_dir = "录音保存目录"
auto_record = true  # 自动录音
min_call_duration = 5  # 最小时长（秒）
```

### 音频质量
```toml
[audio]
sample_rate = 44100  # 采样率 (8000-48000 Hz)
bitrate = 128        # 比特率 (64-320 kbps)
channels = 2         # 声道 (1 或 2)
quality = 2          # 质量 (0-9)
```

### 监控应用
```toml
monitored_apps = [
    "WeChat.exe",
    "QQ.exe",
    "自定义应用.exe",
]
```

---

## 🔧 编译要求

### 必需工具
- Rust 1.70+
- Cargo

### 平台依赖

**Windows**: 无额外依赖

**Linux**:
```bash
sudo apt-get install libasound2-dev pkg-config
```

**macOS**: 无额外依赖

---

## 🤖 GitHub Actions 自动发布

推送标签即可触发自动构建：

```bash
# 创建版本标签
git tag v0.1.0

# 推送标签
git push origin v0.1.0
```

自动构建平台：
- ✅ Windows x64
- ✅ Windows x86
- ✅ macOS Intel
- ✅ macOS Apple Silicon
- ✅ Linux x64

构建完成后自动创建 GitHub Release。

---

## 📝 使用示例

### 示例 1: 自动录制微信通话
```bash
./auto-audio-recorder run
# 打开微信，开始语音通话
# 程序自动检测并开始录音
# 通话结束自动停止并保存
```

### 示例 2: 手动录制会议
```bash
./auto-audio-recorder start
# 开始录音
# ... 进行会议 ...
# Ctrl+C 停止录音
```

### 示例 3: GUI 控制
```bash
./auto-audio-recorder gui
# 使用图形界面控制
# 可实时查看状态
# 可调整设置
```

---

## 🐛 常见问题

### Q: 编译失败？
A: 确保安装了 Rust 1.70+ 和必要的系统依赖。

### Q: 无法录制扬声器？
A: Windows 需要启用"立体声混音"，macOS/Linux 需要额外配置。

### Q: 自动检测不工作？
A: 检查配置文件中的应用名称是否正确。

### Q: 文件太大？
A: 降低比特率或采样率，或使用单声道。

详细解决方案见 `QUICKSTART.md`

---

## 📞 技术支持

- 📖 完整文档: `README.md` 和 `README_ZH.md`
- 🚀 快速开始: `QUICKSTART.md`
- 🔨 编译指南: `BUILD.md`
- 💬 提问反馈: GitHub Issues

---

## 📜 许可证

MIT License - 可自由使用、修改、分发

---

## ✅ 完成清单

- [x] 音频捕获模块（麦克风+扬声器）
- [x] MP3 编码模块（纯 Rust）
- [x] 进程监控模块
- [x] 录音控制模块
- [x] 配置管理模块
- [x] GUI 界面
- [x] 命令行接口
- [x] GitHub Actions 自动发布
- [x] 完整文档（中英文）
- [x] 配置示例和说明
- [x] 快速使用指南
- [x] 编译构建指南

---

**项目已完成！可以直接编译运行！**

```bash
cd audio_recorder
cargo build --release
./target/release/auto-audio-recorder gui
```
