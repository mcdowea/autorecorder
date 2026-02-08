# 快速开始 🚀

## 5 分钟快速上手

### Windows 用户

#### 1️⃣ 下载程序
从 [Releases](https://github.com/yourusername/auto-recorder/releases) 下载 `auto-recorder-windows-x64.exe`

#### 2️⃣ 启用立体声混音
1. 右键点击音量图标 → 声音设置
2. 更多声音设置 → 录制标签
3. 右键空白处 → 显示已禁用的设备
4. 找到"立体声混音" → 右键启用

#### 3️⃣ 生成配置
```cmd
auto-recorder.exe gen-config
```

#### 4️⃣ 开始录音

**自动模式（推荐）：**
```cmd
auto-recorder.exe auto
```
程序会自动检测微信、QQ 等通话并录音

**手动模式：**
```cmd
auto-recorder.exe record
```
按 Ctrl+C 停止录音

#### 5️⃣ 查看录音
录音文件保存在 `recordings` 文件夹中，文件名格式：`recording_20240207_153045.wav`

**可选：转换为 MP3**
```cmd
REM 需要先安装 ffmpeg
convert_to_mp3.bat
```

---

### Linux/macOS 用户

#### 1️⃣ 从源码编译
```bash
# 安装 Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 克隆并编译
git clone https://github.com/yourusername/auto-recorder.git
cd auto-recorder
cargo build --release
```

#### 2️⃣ 生成配置
```bash
./target/release/auto-recorder gen-config
```

#### 3️⃣ 手动录音
```bash
./target/release/auto-recorder record
```

**注意：** Linux/macOS 暂不支持自动录音功能

---

## 常用命令

```bash
# 查看帮助
auto-recorder --help

# 列出音频设备
auto-recorder list-devices

# 高质量录音
auto-recorder record --sample-rate 48000 --bit-rate 320 --quality 0

# 自定义输出目录
auto-recorder record --output ./my-calls

# 启用详细日志
auto-recorder --verbose auto
```

---

## 配置示例

### 高质量通话录音
```json
{
  "sample_rate": 48000,
  "bit_rate": 192,
  "quality": 1,
  "monitored_apps": ["WeChat.exe", "Teams.exe", "Zoom.exe"]
}
```

### 节省空间
```json
{
  "sample_rate": 22050,
  "bit_rate": 64,
  "quality": 7
}
```

---

## 下一步

- 📖 阅读 [完整用户指南](USER_GUIDE_CN.md)
- 🔧 查看 [开发文档](DEVELOPMENT.md)
- ❓ 查看 [常见问题](USER_GUIDE_CN.md#常见问题)
- 🐛 [报告问题](https://github.com/yourusername/auto-recorder/issues)

---

## 重要提醒 ⚠️

录音他人通话前请务必：
1. ✅ 获得所有参与者的明确同意
2. ✅ 遵守当地法律法规
3. ✅ 仅用于合法用途

**使用本软件即表示您同意自行承担所有法律责任。**
