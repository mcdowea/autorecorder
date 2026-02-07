# 自动录音程序

[English](README.md) | 简体中文

一个功能强大的自动录音程序，使用纯 Rust 实现，无需 FFmpeg 和 lame_enc.dll，支持自动检测通话并录音，将音频保存为 MP3 格式。

## ✨ 核心特性

- 🎙️ **双通道同时录制**：同时捕获麦克风和扬声器音频
- 🤖 **智能自动检测**：自动检测微信、QQ、飞书、Skype 等通话软件
- 🎵 **纯 Rust MP3 编码**：无需任何外部依赖库
- ⚙️ **高度可配置**：自定义采样率、比特率、编码质量
- 🖥️ **友好图形界面**：简单易用的 GUI 界面
- 📝 **完整命令行支持**：支持后台运行和脚本自动化
- 🔄 **跨平台支持**：Windows、macOS、Linux 全平台

## 📦 安装

### 方式一：下载预编译版本

从 [Releases](https://github.com/yourusername/auto-audio-recorder/releases) 下载对应平台的可执行文件：

- Windows: `auto-audio-recorder-windows-x64.exe`
- macOS Intel: `auto-audio-recorder-macos-x64`
- macOS Apple Silicon: `auto-audio-recorder-macos-arm64`
- Linux: `auto-audio-recorder-linux-x64`

### 方式二：从源码编译

```bash
# 安装 Rust（如果还没有）
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 克隆仓库
git clone https://github.com/yourusername/auto-audio-recorder.git
cd auto-audio-recorder

# 编译（发布版本）
cargo build --release

# 可执行文件在 target/release/ 目录
```

详细编译说明请查看 [BUILD.md](BUILD.md)

## 🚀 快速开始

### 1️⃣ 图形界面模式（最简单）

```bash
# 直接运行或执行
./auto-audio-recorder

# 或明确指定 GUI 模式
./auto-audio-recorder gui
```

在界面中：
1. 点击"启动录音器"
2. 勾选"启用自动录音"
3. 正常使用微信/QQ 等通话软件
4. 程序自动录音并保存

### 2️⃣ 自动录音模式（推荐）

```bash
# 启动自动检测和录音
./auto-audio-recorder run
```

程序将：
- ✅ 监控微信、QQ、飞书等通话软件
- ✅ 检测到通话时自动开始录音
- ✅ 同时录制麦克风和扬声器
- ✅ 通话结束自动停止并保存为 MP3

### 3️⃣ 手动录音模式

```bash
# 开始手动录音
./auto-audio-recorder start

# 按 Ctrl+C 停止并保存
```

### 4️⃣ 其他命令

```bash
# 查看当前配置
./auto-audio-recorder config

# 列出可用音频设备
./auto-audio-recorder devices

# 查看帮助
./auto-audio-recorder --help
```

## ⚙️ 配置说明

配置文件位置：
- **Windows**: `%APPDATA%\auto-audio-recorder\config.toml`
- **macOS**: `~/Library/Application Support/auto-audio-recorder/config.toml`
- **Linux**: `~/.config/auto-audio-recorder/config.toml`

### 基础配置

```toml
# 录音文件保存目录
output_dir = "C:\\Users\\YourName\\Documents\\AudioRecordings"

# 是否启用自动录音
auto_record = true

# 最小录音时长（秒），少于此时长不保存
min_call_duration = 5

[audio]
# 采样率 (Hz) - 推荐 44100
sample_rate = 44100

# 比特率 (kbps) - 推荐 128
bitrate = 128

# 声道数 (1=单声道, 2=立体声)
channels = 2

# 编码质量 (0-9, 0 最高质量) - 推荐 2
quality = 2

# 监控的通话应用列表
monitored_apps = [
    "WeChat.exe",    # 微信
    "QQ.exe",        # QQ
    "Feishu.exe",    # 飞书
    "Skype.exe",     # Skype
    "Teams.exe",     # Teams
    "Zoom.exe",      # Zoom
    "DingTalk.exe",  # 钉钉
]
```

### 预设配置方案

#### 🎯 高质量录音（重要会议）
```toml
[audio]
sample_rate = 48000
bitrate = 256
channels = 2
quality = 0
```
- 文件大小：约 2 MB/分钟
- 适用场景：重要会议、专业采访

#### 📞 普通通话（推荐）
```toml
[audio]
sample_rate = 44100
bitrate = 128
channels = 2
quality = 2
```
- 文件大小：约 1 MB/分钟
- 适用场景：日常通话、在线会议

#### 💾 节省空间（长时间录音）
```toml
[audio]
sample_rate = 22050
bitrate = 64
channels = 1
quality = 5
```
- 文件大小：约 0.5 MB/分钟
- 适用场景：长时间录音、存储空间有限

完整配置示例请查看 [config.example.toml](config.example.toml)

## 📱 支持的应用

程序默认支持以下通话应用的自动检测：

| 应用类别 | 支持的软件 |
|---------|-----------|
| 即时通讯 | 微信、QQ、TIM |
| 企业协作 | 飞书、钉钉、企业微信 |
| 视频会议 | Zoom、Teams、Skype、腾讯会议 |

您可以在配置文件的 `monitored_apps` 中添加其他应用。

## 🛠️ 核心技术

- **音频捕获**: [cpal](https://github.com/RustAudio/cpal) - 跨平台音频 I/O 库
- **MP3 编码**: [mp3lame-encoder](https://github.com/nfam/lame.rs) - 纯 Rust LAME 编码器
- **异步运行**: [Tokio](https://tokio.rs/) - 异步运行时
- **图形界面**: [egui](https://github.com/emilk/egui) - 即时模式 GUI
- **进程监控**: [sysinfo](https://github.com/GuillaumeGomez/sysinfo) - 系统信息获取

## 🎬 使用场景

### 场景一：远程会议录音
```bash
# 启动自动模式
./auto-audio-recorder run
```
打开 Teams/Zoom，开始会议，程序自动录音。

### 场景二：客服通话录音
```bash
# GUI 模式，手动控制
./auto-audio-recorder gui
```
接听电话前点击"开始录音"，通话结束点击"停止"。

### 场景三：播客录制
```bash
# 手动模式，高质量设置
./auto-audio-recorder start
```
配置高质量参数，手动控制录音过程。

### 场景四：自动备份通话
后台运行，所有通话自动保存：
```bash
# Linux/macOS 后台运行
nohup ./auto-audio-recorder run > recorder.log 2>&1 &

# Windows 可以设置为系统服务
```

## 🔧 常见问题

### Q1: 无法录制扬声器声音？

**Windows 解决方案**：
1. 右键任务栏音量图标 → 声音设置
2. 在"输入设备"中找到"立体声混音"
3. 启用立体声混音设备

**macOS 解决方案**：
- 需要安装虚拟音频设备（如 BlackHole）
- 或使用音频路由软件（如 Loopback）

**Linux 解决方案**：
```bash
# 安装并配置 PulseAudio
sudo apt install pulseaudio pavucontrol
pavucontrol  # 打开音频配置工具
```

### Q2: 自动录音不工作？

检查清单：
- [ ] 配置中 `auto_record = true`
- [ ] 目标应用在 `monitored_apps` 列表中
- [ ] 应用名称正确（使用任务管理器确认）
- [ ] 以 `run` 模式启动程序

### Q3: 录音文件太大？

优化配置：
```toml
[audio]
sample_rate = 22050  # 降低采样率
bitrate = 64         # 降低比特率
channels = 1         # 单声道
quality = 5          # 较低质量
```

### Q4: 录音质量不够好？

提升配置：
```toml
[audio]
sample_rate = 48000  # 提高采样率
bitrate = 192        # 提高比特率
channels = 2         # 立体声
quality = 0          # 最高质量
```

### Q5: 如何添加新的监控应用？

编辑配置文件，添加应用名称：
```toml
monitored_apps = [
    # ... 现有应用 ...
    "YourApp.exe",  # 添加你的应用
]
```

## 📁 文件说明

```
auto-audio-recorder/
├── src/
│   ├── main.rs              # 程序入口
│   ├── config.rs            # 配置管理
│   ├── audio_capture.rs     # 音频捕获
│   ├── encoder.rs           # MP3 编码
│   ├── recorder.rs          # 录音核心
│   ├── process_monitor.rs   # 进程监控
│   └── gui.rs              # GUI 界面
├── Cargo.toml               # 项目配置
├── README.md                # 英文说明
├── README_ZH.md             # 中文说明
├── BUILD.md                 # 编译指南
├── QUICKSTART.md            # 快速开始
└── config.example.toml      # 配置示例
```

## 🤝 参与贡献

欢迎提交问题和改进建议！

1. Fork 本仓库
2. 创建特性分支 (`git checkout -b feature/AmazingFeature`)
3. 提交改动 (`git commit -m '添加某个特性'`)
4. 推送到分支 (`git push origin feature/AmazingFeature`)
5. 创建 Pull Request

## ⚠️ 重要提示

### 隐私和法律

- ⚖️ **法律合规**: 在某些地区，未经许可录音可能违法
- 🔒 **隐私保护**: 录音前请获得所有参与者的明确同意
- 📋 **使用规范**: 仅将录音用于合法、正当的目的
- 🗑️ **安全存储**: 妥善保管录音文件，定期清理

### 使用建议

- 💡 建议在录音前告知对方
- 🔐 重要录音请加密存储
- 📆 定期清理过期录音
- 🆘 遇到问题请查看文档或提交 Issue

## 📄 许可证

本项目采用 MIT 许可证 - 详见 [LICENSE](LICENSE) 文件

## 🙏 致谢

特别感谢以下开源项目：
- LAME MP3 编码器项目
- Rust 音频工作组
- 所有依赖库的维护者和贡献者

## 📮 联系方式

- 💬 GitHub Issues: [提交问题](https://github.com/yourusername/auto-audio-recorder/issues)
- 📧 Email: your.email@example.com
- 🌐 项目主页: [GitHub](https://github.com/yourusername/auto-audio-recorder)

---

**免责声明**: 本软件仅供学习和合法用途。使用者须遵守当地法律法规，自行承担使用本软件产生的所有法律责任。作者不对任何误用、滥用或违法使用行为负责。
