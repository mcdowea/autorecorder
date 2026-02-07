# 快速使用指南

## ⚡ 最快开始方式

### 1. 编译（推荐无 GUI 模式）

```bash
# Windows
cargo build --release --no-default-features --target x86_64-pc-windows-msvc

# 或使用自动脚本
build.bat
```

### 2. 运行

```bash
# 自动录音模式（最常用）
.\target\x86_64-pc-windows-msvc\release\auto-audio-recorder.exe run
```

程序会：
1. ✅ 自动检测微信、QQ、飞书等通话软件
2. ✅ 通话时自动开始录音
3. ✅ 同时录制麦克风和扬声器
4. ✅ 通话结束自动保存为 MP3

录音文件默认保存在: `文档\AudioRecordings\`

---

## 📝 所有命令

```bash
# 自动录音（后台运行）
auto-audio-recorder.exe run

# 手动录音（按 Ctrl+C 停止）
auto-audio-recorder.exe start

# 查看配置
auto-audio-recorder.exe config

# 列出音频设备
auto-audio-recorder.exe devices

# 显示帮助
auto-audio-recorder.exe --help
```

---

## ⚙️ 配置文件

位置: `%APPDATA%\auto-audio-recorder\config.toml`

### 常用配置

```toml
# 自动录音
auto_record = true

# 最小录音时长（秒）
min_call_duration = 5

[audio]
# 采样率（Hz）
sample_rate = 44100

# 比特率（kbps）
bitrate = 128

# 声道数（1=单声道, 2=立体声）
channels = 2

# 质量（0-9, 0 最高质量）
quality = 2
```

### 音质预设

**高质量**（重要会议）:
```toml
[audio]
sample_rate = 48000
bitrate = 192
quality = 0
```

**标准质量**（日常使用）:
```toml
[audio]
sample_rate = 44100
bitrate = 128
quality = 2
```

**节省空间**（长时间录音）:
```toml
[audio]
sample_rate = 22050
bitrate = 64
quality = 5
```

---

## 🔧 常见问题

### 问题 1: 听不到扬声器声音

**解决方案**: 启用立体声混音

1. 右键任务栏音量图标
2. 声音设置 → 声音控制面板
3. 录制选项卡
4. 右键空白处 → 显示已禁用的设备
5. 启用"立体声混音"

### 问题 2: 自动录音不工作

**检查清单**:
1. 配置中 `auto_record = true`
2. 应用在监控列表中
3. 使用 `run` 模式启动

### 问题 3: 编译错误

**使用无 GUI 模式**:
```bash
cargo build --release --no-default-features
```

详细故障排除请查看 [TROUBLESHOOTING.md](TROUBLESHOOTING.md)

---

## 🎯 使用场景

### 场景 1: 微信通话录音
```bash
# 1. 启动程序
auto-audio-recorder.exe run

# 2. 正常使用微信通话
# 3. 通话结束后自动保存
```

### 场景 2: 在线会议录音
```bash
# 支持 Teams、Zoom、腾讯会议等
auto-audio-recorder.exe run
```

### 场景 3: 手动录制播客
```bash
auto-audio-recorder.exe start
# 录制完成按 Ctrl+C
```

---

## 💡 进阶技巧

### 后台运行

创建任务计划程序：
1. Win+R → `taskschd.msc`
2. 创建基本任务
3. 设置触发器（登录时）
4. 操作: 启动程序 `auto-audio-recorder.exe run`

### 修改输出目录

编辑配置文件:
```toml
output_dir = "D:\\Recordings"
```

### 添加监控应用

```toml
monitored_apps = [
    "WeChat.exe",
    "QQ.exe",
    "YourApp.exe",  # 添加你的应用
]
```

---

## 📋 编译选项

### 无 GUI 版本（推荐）

```bash
cargo build --release --no-default-features
```

优点:
- ✅ 编译快（3-5 分钟）
- ✅ 体积小（~10 MB）
- ✅ 无兼容性问题

### 带 GUI 版本

```bash
cargo build --release --features gui
```

优点:
- ✅ 图形界面
- ✅ 实时状态显示

详细说明: [COMPILE_OPTIONS.md](COMPILE_OPTIONS.md)

---

## 🆘 获取帮助

- 📖 详细文档: [README_ZH.md](README_ZH.md)
- 🔧 编译指南: [BUILD.md](BUILD.md)
- 🐛 故障排除: [TROUBLESHOOTING.md](TROUBLESHOOTING.md)
- 💬 提交问题: GitHub Issues

---

**提示**: 首次使用建议先测试手动录音，确保设备正常后再使用自动模式。
