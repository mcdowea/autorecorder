# Auto Audio Recorder (Windows Only)

[![Build and Release](https://github.com/yourusername/auto-audio-recorder/actions/workflows/release.yml/badge.svg)](https://github.com/yourusername/auto-audio-recorder/actions/workflows/release.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A powerful automatic audio recording program implemented in pure Rust, supports automatic detection of calls and records audio in MP3 format. **Windows Only**

## ✨ Main Features

- 🎙️ **Dual Channel Recording**: Simultaneously record microphone and speaker audio
- 🤖 **Auto Detection**: Automatically detect WeChat, QQ, Lark, Skype and other call software
- 🎵 **MP3 Encoding**: Pure Rust MP3 encoding, no external dependencies needed
- ⚙️ **Highly Configurable**: Customize sample rate, bitrate, quality and other parameters
- 🖥️ **Graphical Interface**: Easy-to-use GUI interface
- 📝 **Command Line Support**: Support background running and manual control
- 🪟 **Windows Platform**: Optimized for Windows with WASAPI Loopback support

## 🚀 Quick Start

### Installation

#### Download from Releases

Visit [Releases](https://github.com/yourusername/auto-audio-recorder/releases) to download the pre-compiled binary:

- Windows x64: `auto-audio-recorder-windows-x64.exe`
- Windows x86: `auto-audio-recorder-windows-x86.exe`

#### Build from Source

```bash
# Clone repository
git clone https://github.com/yourusername/auto-audio-recorder.git
cd auto-audio-recorder

# Build
cargo build --release

# The compiled program is in target/release/auto-audio-recorder.exe
```

### 使用方法

#### 1. GUI 模式（推荐）

```bash
# 启动图形界面
./auto-audio-recorder gui

# 或直接运行（默认启动 GUI）
./auto-audio-recorder
```

#### 2. 自动录音模式

```bash
# 后台运行，自动检测并录音
./auto-audio-recorder run

# 禁用自动录音（仅启动录音器）
./auto-audio-recorder run --no-auto
```

#### 3. 手动录音

```bash
# 开始录音，按 Ctrl+C 停止
./auto-audio-recorder start
```

#### 4. 查看配置

```bash
# 显示当前配置
./auto-audio-recorder config

# 列出音频设备
./auto-audio-recorder devices
```

## ⚙️ 配置

配置文件位于：
- **Windows**: `%APPDATA%\auto-audio-recorder\config.toml`
- **macOS**: `~/Library/Application Support/auto-audio-recorder/config.toml`
- **Linux**: `~/.config/auto-audio-recorder/config.toml`

### 配置示例

```toml
output_dir = "C:\\Users\\YourName\\Documents\\AudioRecordings"
auto_record = true
min_call_duration = 5

[audio]
sample_rate = 44100
bitrate = 128
channels = 2
quality = 2

monitored_apps = [
    "WeChat.exe",
    "QQ.exe",
    "Lark.exe",
    "Feishu.exe",
    "Skype.exe",
    "Teams.exe",
    "Zoom.exe",
    "DingTalk.exe"
]
```

### 配置说明

| 参数 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `output_dir` | 字符串 | 文档/AudioRecordings | 录音文件保存目录 |
| `auto_record` | 布尔值 | true | 是否启用自动录音 |
| `min_call_duration` | 整数 | 5 | 最小通话时长（秒），少于此时长不保存 |
| `audio.sample_rate` | 整数 | 44100 | 采样率 (Hz) |
| `audio.bitrate` | 整数 | 128 | 比特率 (kbps) |
| `audio.channels` | 整数 | 2 | 声道数 (1=单声道, 2=立体声) |
| `audio.quality` | 整数 | 2 | MP3 质量 (0-9, 0 为最高质量) |
| `monitored_apps` | 数组 | [...] | 要监控的应用程序列表 |

## 🎯 支持的应用

默认支持以下通话应用的自动检测：

- 微信 (WeChat)
- QQ
- 飞书 (Lark/Feishu)
- Skype
- Microsoft Teams
- Zoom
- 钉钉 (DingTalk)

您可以在配置文件中添加更多应用。

## 🛠️ 技术栈

- **音频捕获**: [cpal](https://github.com/RustAudio/cpal)
- **MP3 编码**: [mp3lame-encoder](https://github.com/nfam/lame.rs)
- **异步运行时**: [Tokio](https://tokio.rs/)
- **GUI 框架**: [egui](https://github.com/emilk/egui)
- **进程监控**: [sysinfo](https://github.com/GuillaumeGomez/sysinfo)

## 📋 系统要求

### Windows
- Windows 10 或更高版本
- 支持 WASAPI 的音频驱动

### macOS
- macOS 10.12 或更高版本

### Linux
- ALSA 或 PulseAudio

## 🔧 开发

### 构建要求

- Rust 1.70 或更高版本
- Cargo

### 编译

```bash
# 开发构建
cargo build

# 发布构建
cargo build --release

# 运行测试
cargo test
```

### 代码结构

```
src/
├── main.rs              # 主入口
├── config.rs            # 配置管理
├── audio_capture.rs     # 音频捕获
├── encoder.rs           # MP3 编码
├── recorder.rs          # 录音器核心
├── process_monitor.rs   # 进程监控
└── gui.rs              # GUI 界面
```

## 📝 许可证

本项目采用 MIT 许可证。详见 [LICENSE](LICENSE) 文件。

## 🤝 贡献

欢迎提交 Issue 和 Pull Request！

1. Fork 本仓库
2. 创建特性分支 (`git checkout -b feature/AmazingFeature`)
3. 提交更改 (`git commit -m 'Add some AmazingFeature'`)
4. 推送到分支 (`git push origin feature/AmazingFeature`)
5. 开启 Pull Request

## ⚠️ 注意事项

1. **隐私**: 录音功能可能涉及隐私问题，请确保在录音前获得所有相关方的同意
2. **法律**: 在某些地区，未经许可录音可能违法，请遵守当地法律法规
3. **资源**: 长时间录音会占用磁盘空间，请定期清理旧文件
4. **权限**: 某些系统需要授予麦克风和音频录制权限

## 🐛 已知问题

- Linux 下扬声器捕获可能需要额外配置 PulseAudio
- macOS 可能需要在系统偏好设置中授予麦克风权限

## 📮 联系方式

如有问题或建议，请通过以下方式联系：

- 提交 [Issue](https://github.com/yourusername/auto-audio-recorder/issues)
- 发送邮件至: your.email@example.com

## 🙏 致谢

感谢所有开源项目的贡献者，特别是：

- LAME MP3 编码器团队
- Rust 音频社区
- 所有依赖库的维护者

---

**免责声明**: 本软件仅供学习和合法用途使用。使用者需自行承担使用本软件的所有法律责任。
