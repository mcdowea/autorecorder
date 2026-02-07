# Auto Recorder 🎙️

自动录音程序，支持同时录制麦克风和扬声器音频，检测通话应用并自动开始/停止录音。

## ✨ 功能特性

- 🎤 **双音频源录制**：同时捕获麦克风和扬声器音频并混合
- 🤖 **智能自动录音**：检测微信、QQ、飞书、Skype 等通话应用，自动开始/停止录音
- 📝 **手动录音模式**：支持按需手动录音
- 🎵 **WAV 格式录音**：高质量无损音频，可选转换为 MP3
- ⚙️ **高度可配置**：自定义采样率等参数
- 🔇 **静音检测**：自动检测静音并停止录音
- 💾 **自动保存**：录音自动保存为带时间戳的 WAV 文件
- 🔄 **MP3 转换**：提供便捷的 WAV 转 MP3 脚本

## 🚀 快速开始

### 安装依赖

**Windows 用户需要先启用"立体声混音"设备：**

1. 右键点击任务栏音量图标 → 声音设置
2. 高级 → 更多声音设置
3. 录制标签页
4. 右键空白处 → 显示已禁用的设备
5. 找到"立体声混音"或"Stereo Mix"
6. 右键 → 启用

### 构建

```bash
# 克隆仓库
git clone https://github.com/yourusername/auto-recorder.git
cd auto-recorder

# 构建
cargo build --release

# 运行
./target/release/auto-recorder --help
```

## 📖 使用方法

### 1. 生成默认配置文件

```bash
auto-recorder gen-config
```

这将创建 `config.json` 文件，内容如下：

```json
{
  "output_dir": "recordings",
  "sample_rate": 44100,
  "bit_rate": 128,
  "quality": 2,
  "auto_recording": true,
  "monitored_apps": [
    "WeChat.exe",
    "QQ.exe",
    "Lark.exe",
    "feishu.exe",
    "Skype.exe",
    "Teams.exe",
    "Zoom.exe",
    "Discord.exe"
  ],
  "silence_threshold": 0.01,
  "silence_duration": 3
}
```

### 2. 列出音频设备

```bash
auto-recorder list-devices
```

### 3. 自动录音模式

```bash
# 使用默认配置
auto-recorder auto

# 使用自定义配置
auto-recorder --config my-config.json auto

# 启用详细日志
auto-recorder --verbose auto
```

程序将监控指定的通话应用，当检测到通话时自动开始录音，通话结束后自动停止。

### 4. 手动录音模式

```bash
# 使用默认设置
auto-recorder record

# 自定义参数
auto-recorder record --sample-rate 48000 --bit-rate 192 --quality 0

# 指定输出目录
auto-recorder record --output ./my-recordings

# 按 Ctrl+C 停止录音
```

### 5. 转换 WAV 为 MP3（可选）

录音默认保存为 WAV 格式。如需 MP3 格式，可使用提供的转换脚本：

**Linux/macOS:**
```bash
# 确保已安装 ffmpeg
sudo apt-get install ffmpeg  # Ubuntu/Debian
brew install ffmpeg          # macOS

# 转换所有 WAV 文件
chmod +x convert_to_mp3.sh
./convert_to_mp3.sh

# 自定义参数
./convert_to_mp3.sh -b 192k -q 1  # 高质量 MP3
```

**Windows:**
```cmd
REM 确保已安装 ffmpeg 并添加到 PATH

REM 转换所有 WAV 文件
convert_to_mp3.bat

REM 自定义参数
convert_to_mp3.bat -b 192k -q 1
```

**转换参数说明：**
- `-b` / `--bitrate`: MP3 比特率 (64k, 128k, 192k, 320k)
- `-q` / `--quality`: MP3 质量 (0-9, 0最好)
- `-i` / `--input`: 输入目录
- `-o` / `--output`: 输出目录

## ⚙️ 配置说明

| 参数 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `output_dir` | 字符串 | `"recordings"` | 录音文件保存目录 |
| `sample_rate` | 整数 | `44100` | 采样率 (Hz)，推荐：44100 或 48000 |
| `bit_rate` | 整数 | `128` | 保留字段（WAV格式不使用） |
| `quality` | 整数 | `2` | 保留字段（WAV格式不使用） |
| `auto_recording` | 布尔 | `true` | 是否启用自动录音 |
| `monitored_apps` | 数组 | 见上文 | 要监控的应用程序列表 |
| `silence_threshold` | 浮点 | `0.01` | 静音检测阈值 (0.0-1.0) |
| `silence_duration` | 整数 | `3` | 静音持续多少秒后停止录音 |

## 🛠️ 技术栈

- **音频捕获**：[cpal](https://github.com/RustAudio/cpal) - 跨平台音频 I/O
- **音频编码**：[hound](https://github.com/ruuda/hound) - WAV 文件读写
- **进程监控**：Windows API (仅 Windows)
- **异步运行时**：[tokio](https://tokio.rs/)
- **命令行解析**：[clap](https://github.com/clap-rs/clap)
- **MP3 转换**：外部 ffmpeg 工具（可选）

## 📋 系统要求

- **操作系统**：Windows 10/11（自动录音功能）, Linux, macOS（仅手动录音）
- **Rust**：1.70+
- **音频设备**：需要麦克风和扬声器/耳机

## 🐛 故障排除

### Windows 上无法录制扬声器音频

确保已启用"立体声混音"设备（参见上文安装步骤）。

### 录音文件为空或很小

1. 检查音频设备是否正常工作
2. 调整 `silence_threshold` 参数
3. 使用 `--verbose` 查看详细日志

### 检测不到通话应用

1. 确认应用名称在 `monitored_apps` 列表中
2. 使用任务管理器查看进程名称是否正确
3. 某些应用可能使用不同的进程名

## 📄 许可证

MIT License

## 🤝 贡献

欢迎提交 Issue 和 Pull Request！

## ⚠️ 免责声明

本软件仅供学习和个人使用。录音他人通话可能违反当地法律法规，请确保：

1. 获得所有参与者的明确同意
2. 遵守当地隐私和录音相关法律
3. 不将录音用于非法目的

使用本软件即表示您同意自行承担所有法律责任。
