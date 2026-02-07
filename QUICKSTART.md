# 快速使用指南

## 🚀 5 分钟快速上手

### 第一步：安装程序

#### Windows 用户
1. 从 [Releases](https://github.com/yourusername/auto-audio-recorder/releases) 下载 `auto-audio-recorder-windows-x64.exe`
2. 双击运行即可

#### macOS 用户
1. 下载 `auto-audio-recorder-macos-x64`（Intel）或 `auto-audio-recorder-macos-arm64`（Apple Silicon）
2. 打开终端，运行：
   ```bash
   chmod +x auto-audio-recorder-macos-*
   ./auto-audio-recorder-macos-*
   ```

#### Linux 用户
```bash
chmod +x auto-audio-recorder-linux-x64
./auto-audio-recorder-linux-x64
```

### 第二步：首次运行

1. **启动程序**
   ```bash
   # GUI 模式（推荐）
   ./auto-audio-recorder
   
   # 或者
   ./auto-audio-recorder gui
   ```

2. **授予权限**（如需要）
   - Windows: 允许防火墙访问
   - macOS: 系统偏好设置 → 安全性与隐私 → 麦克风
   - Linux: 确保用户在 audio 组

3. **检查设置**
   - 输出目录：默认在「文档/AudioRecordings」
   - 自动录音：默认开启
   - 音质设置：默认 44.1kHz, 128kbps

### 第三步：开始使用

#### 方式一：自动录音（推荐）

```bash
# 启动自动录音模式
./auto-audio-recorder run
```

- ✅ 打开微信、QQ、飞书等通话应用
- ✅ 接听或拨打电话
- ✅ 程序自动开始录音
- ✅ 通话结束自动停止并保存

#### 方式二：手动录音

```bash
# 开始手动录音
./auto-audio-recorder start

# 按 Ctrl+C 停止录音
```

#### 方式三：GUI 控制

1. 启动 GUI 界面
2. 点击「启动录音器」
3. 启用「自动录音」或点击「开始录音」
4. 完成后点击「停止录音」

---

## 📋 常见使用场景

### 场景 1：录制微信语音通话

```bash
# 方法一：自动模式
./auto-audio-recorder run

# 方法二：GUI 模式
./auto-audio-recorder gui
# 然后点击"启动录音器"，启用"自动录音"
```

使用步骤：
1. 启动程序（自动检测微信）
2. 正常使用微信语音/视频通话
3. 通话结束后，录音自动保存到输出目录

### 场景 2：录制在线会议

支持的会议软件：
- Microsoft Teams
- Zoom
- 腾讯会议
- 飞书/Lark

```bash
./auto-audio-recorder run
```

程序会自动检测会议软件并开始录音。

### 场景 3：录制播客或音频内容

```bash
# 手动开始录音
./auto-audio-recorder start

# 录制完成后按 Ctrl+C
```

### 场景 4：定制化录音

1. 编辑配置文件（参考下一节）
2. 调整音质参数
3. 运行程序

---

## ⚙️ 配置修改

### 查看当前配置

```bash
./auto-audio-recorder config
```

### 配置文件位置

- **Windows**: `%APPDATA%\auto-audio-recorder\config.toml`
- **macOS**: `~/Library/Application Support/auto-audio-recorder/config.toml`
- **Linux**: `~/.config/auto-audio-recorder/config.toml`

### 快速配置示例

#### 高质量录音（会议、采访）
```toml
[audio]
sample_rate = 48000
bitrate = 256
channels = 2
quality = 0
```

#### 普通通话录音（日常使用）
```toml
[audio]
sample_rate = 44100
bitrate = 128
channels = 2
quality = 2
```

#### 节省空间（长时间录音）
```toml
[audio]
sample_rate = 22050
bitrate = 64
channels = 1
quality = 5
```

### 修改输出目录

```toml
# Windows
output_dir = "D:\\Recordings"

# macOS/Linux
output_dir = "/home/username/Recordings"
```

### 添加监控应用

```toml
monitored_apps = [
    "WeChat.exe",
    "YourApp.exe",  # 添加您的应用
]
```

---

## 🎛️ 命令行参数

### 显示帮助
```bash
./auto-audio-recorder --help
```

### 查看版本
```bash
./auto-audio-recorder --version
```

### 列出音频设备
```bash
./auto-audio-recorder devices
```

输出示例：
```
可用音频设备:
  输入: 麦克风 (Realtek High Definition Audio)
  输入: 立体声混音
  输出: 扬声器 (Realtek High Definition Audio)
  输出: 耳机
```

---

## 🔍 故障排查

### 问题 1：没有检测到音频设备

**解决方法**：
```bash
# 列出设备
./auto-audio-recorder devices

# 检查系统音频设置
# Windows: 控制面板 → 声音
# macOS: 系统偏好设置 → 声音
# Linux: 音频设置
```

### 问题 2：无法录制扬声器声音

**Windows 解决方法**：
1. 右键点击任务栏音量图标
2. 选择"声音设置"
3. 在"输入"中启用"立体声混音"

**macOS 解决方法**：
- macOS 需要使用 BlackHole 或 Loopback 等虚拟音频设备

**Linux 解决方法**：
```bash
# 安装 PulseAudio
sudo apt-get install pulseaudio pavucontrol

# 使用 pavucontrol 配置音频
pavucontrol
```

### 问题 3：自动录音不工作

**检查清单**：
1. ✓ 配置中 `auto_record = true`
2. ✓ 监控的应用在 `monitored_apps` 列表中
3. ✓ 应用程序名称正确（使用任务管理器查看）
4. ✓ 程序以 `run` 模式启动

### 问题 4：录音文件太大

**优化方法**：
```toml
[audio]
sample_rate = 22050  # 降低采样率
bitrate = 64         # 降低比特率
channels = 1         # 使用单声道
quality = 5          # 提高质量值（文件更小）
```

### 问题 5：录音质量不佳

**提升方法**：
```toml
[audio]
sample_rate = 48000  # 提高采样率
bitrate = 192        # 提高比特率
channels = 2         # 使用立体声
quality = 0          # 最高质量
```

---

## 📁 文件管理

### 录音文件命名规则

格式：`recording_YYYYMMDD_HHMMSS.mp3`

示例：`recording_20241207_143025.mp3`

### 批量管理录音

```bash
# 查看所有录音
ls ~/Documents/AudioRecordings/

# 按日期筛选
ls ~/Documents/AudioRecordings/recording_20241207*

# 删除旧录音（谨慎操作）
find ~/Documents/AudioRecordings/ -name "*.mp3" -mtime +30 -delete
```

---

## 💡 使用技巧

### 技巧 1：后台运行

```bash
# Linux/macOS
nohup ./auto-audio-recorder run > recorder.log 2>&1 &

# Windows (使用 Task Scheduler 或服务)
```

### 技巧 2：开机自启动

**Windows**：
1. Win+R 输入 `shell:startup`
2. 创建快捷方式到该文件夹

**Linux (systemd)**：
```bash
# 创建服务文件
sudo nano /etc/systemd/system/audio-recorder.service

# 添加内容
[Unit]
Description=Auto Audio Recorder
After=network.target

[Service]
ExecStart=/path/to/auto-audio-recorder run
Restart=always

[Install]
WantedBy=multi-user.target

# 启用服务
sudo systemctl enable audio-recorder
sudo systemctl start audio-recorder
```

### 技巧 3：定期清理

建议设置自动清理脚本：

```bash
#!/bin/bash
# 删除 30 天前的录音
find ~/Documents/AudioRecordings/ -name "*.mp3" -mtime +30 -delete
```

添加到 crontab：
```bash
# 每天凌晨 2 点执行
0 2 * * * /path/to/cleanup.sh
```

---

## 🆘 获取帮助

- 📖 查看完整文档：[README.md](README.md)
- 🐛 报告问题：[GitHub Issues](https://github.com/yourusername/auto-audio-recorder/issues)
- 💬 讨论交流：[GitHub Discussions](https://github.com/yourusername/auto-audio-recorder/discussions)

---

**提示**：首次使用建议先用手动模式测试，确保录音正常后再启用自动模式。
