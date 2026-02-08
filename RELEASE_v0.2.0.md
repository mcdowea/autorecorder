# Auto Recorder v0.2.0 - GUI 版本发布！

## 🎉 重大更新

Auto Recorder v0.2.0 新增了**完整的图形用户界面**！现在您可以选择：

- 📱 **GUI 版本** - 现代化图形界面，双击即用
- 💻 **CLI 版本** - 命令行工具，灵活强大

## ✨ 新功能预览

### 主界面设计

```
┌──────────────────────────────────────────┐
│                                          │
│   00:00:00.0    [开始] [停止]  [查看]   │
│                                          │
│   当前模式：语音视频聊天自动录音。       │
│                                          │
│   🎤 ████████████░░░░░░░░░░░            │
│   🔊 ██████████████░░░░░░░░░            │
│                                          │
│   ⚙ 设置                                │
│                                          │
└──────────────────────────────────────────┘
```

### 核心特性

1. ⏱️ **实时计时器**
   - 精确到 0.1 秒
   - HH:MM:SS.D 格式显示
   - 录音时持续更新

2. 📊 **音频可视化**
   - 麦克风输入电平实时显示
   - 扬声器输出电平实时显示
   - 渐变色彩（绿→黄→红）

3. 🎯 **三种录音模式**
   - **手动录音** - 完全手动控制
   - **自动录音** - 软件启动即开始
   - **程序检测** - 监控微信/QQ等通话应用

4. ⚙️ **全功能设置**
   - 录音模式切换
   - 音频源选择
   - 保存路径设置
   - 采样率/比特率配置
   - 监控应用自定义

5. 🖱️ **一键操作**
   - 开始录音
   - 停止录音
   - 查看文件夹
   - 打开设置

## 🚀 快速开始

### 方法 1：使用 GUI（推荐）

1. 下载 `auto-recorder-gui.exe`
2. 双击运行
3. 点击"开始"按钮
4. 开始录音！

### 方法 2：使用 CLI

1. 下载 `auto-recorder.exe`
2. 双击 `launcher.bat`
3. 选择菜单选项
4. 按提示操作

## 📦 下载文件

### Windows 用户

#### GUI 版本（图形界面）
- `auto-recorder-gui-windows-x64.exe` - 64位版本
- `auto-recorder-gui-windows-x86.exe` - 32位版本

#### CLI 版本（命令行）
- `auto-recorder-windows-x64.exe` - 64位版本
- `auto-recorder-windows-x86.exe` - 32位版本

### Linux/macOS 用户

- `auto-recorder-linux-x64` - Linux 64位
- `auto-recorder-macos-x64` - macOS Intel
- `auto-recorder-macos-arm64` - macOS Apple Silicon

*注：Linux/macOS 暂无 GUI 版本，仅支持 CLI*

## 💡 使用建议

### 选择 GUI 版本如果：
- ✓ 你是普通用户
- ✓ 喜欢可视化界面
- ✓ 需要实时查看录音状态
- ✓ 不熟悉命令行

### 选择 CLI 版本如果：
- ✓ 你是高级用户
- ✓ 需要脚本自动化
- ✓ 作为后台服务运行
- ✓ 在Linux/macOS上使用

## 🔧 系统要求

### 最低配置
- Windows 7 或更高版本
- 2 GB RAM
- 100 MB 可用空间

### 推荐配置
- Windows 10/11
- 4 GB RAM
- 支持立体声混音的声卡

### 必需软件
- Visual C++ Redistributable
  [下载](https://aka.ms/vs/17/release/vc_redist.x64.exe)

## 📋 完整功能列表

### 录音功能
- [x] 麦克风录音
- [x] 扬声器录音（需启用立体声混音）
- [x] 混合录音（麦克风+扬声器）
- [x] WAV 格式输出
- [x] 可配置采样率（8K-48K Hz）
- [x] 可配置比特率（32-320 kbps）

### 自动化功能
- [x] 进程监控（微信、QQ、Teams等）
- [x] 自动开始录音
- [x] 自动停止录音
- [x] 静音检测
- [x] 计划任务（GUI版本）

### 文件管理
- [x] 自动命名（时间戳）
- [x] 自定义保存路径
- [x] 月份文件夹
- [x] 日期文件夹
- [x] WAV 转 MP3 工具

### 用户界面
- [x] 图形界面（GUI）
- [x] 命令行界面（CLI）
- [x] 交互式菜单（CLI）
- [x] 完整中文支持

## 📖 文档资源

- `GUI_GUIDE.md` - GUI 版本完整指南
- `README_WINDOWS.md` - Windows 用户指南
- `USER_GUIDE_CN.md` - 详细中文文档
- `QUICKSTART.md` - 5分钟快速上手
- `WINDOWS_FIX.md` - 常见问题解决

## 🐛 已知问题

1. **GUI 版本仅支持 Windows**
   - Linux/macOS GUI 版本正在开发中

2. **需要启用立体声混音**
   - Windows 用户需手动启用此设备

3. **录音格式为 WAV**
   - 文件较大，可使用转换脚本转为 MP3

4. **自动录音仅限 Windows**
   - 依赖 Windows API 进行进程监控

## 🔮 未来计划

### v0.3.0
- [ ] Linux/macOS GUI 版本
- [ ] 热键控制
- [ ] 波形可视化
- [ ] 内置 MP3 编码

### v1.0.0
- [ ] 多语言支持
- [ ] 云端备份
- [ ] 语音转文字
- [ ] 智能摘要

## 🙏 反馈与支持

### 报告问题
- GitHub Issues: [提交问题](https://github.com/yourusername/auto-recorder/issues)

### 功能建议
- GitHub Discussions: [参与讨论](https://github.com/yourusername/auto-recorder/discussions)

### 获取帮助
- 查看文档：`GUI_GUIDE.md`, `USER_GUIDE_CN.md`
- 运行诊断：`diagnose.bat`
- 联系支持：见项目主页

## 🎁 特别鸣谢

感谢所有使用和支持 Auto Recorder 的用户！

---

**立即下载体验：**
- GUI 版本：`auto-recorder-gui.exe`
- CLI 版本：`auto-recorder.exe`

**享受全新的录音体验！🎙️**
