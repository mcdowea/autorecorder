# Windows 用户使用指南

## ⚠️ 重要：双击 .exe 没反应？

这是**正常现象**！`auto-recorder.exe` 是一个**命令行程序**，需要通过以下方式运行：

## 🚀 推荐方式：使用图形启动器

### 方法 1：双击 `launcher.bat`（最简单）

1. 找到 `launcher.bat` 文件
2. 双击运行
3. 按照菜单选择操作

```
========================================
    Auto Recorder - 自动录音程序
========================================

请选择操作：

  [1] 生成配置文件
  [2] 查看音频设备列表
  [3] 手动录音
  [4] 自动录音（检测通话）
  [5] 转换 WAV 为 MP3
  [6] 测试程序是否正常
  [0] 退出

========================================
```

## 🔧 方法 2：使用命令提示符

### 打开命令提示符的方法：

**方法 A：在文件夹中打开**
1. 打开程序所在文件夹
2. 在地址栏输入 `cmd` 并按回车
3. 自动打开命令提示符并定位到当前目录

**方法 B：使用开始菜单**
1. 按 `Win + R`
2. 输入 `cmd` 并回车
3. 使用 `cd` 命令导航到程序目录：
   ```cmd
   cd C:\path\to\auto-recorder
   ```

### 运行命令：

```cmd
:: 查看帮助
auto-recorder.exe --help

:: 生成配置文件
auto-recorder.exe gen-config

:: 查看音频设备
auto-recorder.exe list-devices

:: 开始录音（手动）
auto-recorder.exe record

:: 自动录音模式
auto-recorder.exe auto
```

## 🔍 诊断工具

如果遇到问题，双击运行 `diagnose.bat`：

```
双击 → diagnose.bat
```

这会自动检查：
- ✓ 程序文件是否存在
- ✓ 程序能否正常启动
- ✓ Visual C++ 运行时
- ✓ 音频设备
- ✓ 配置文件

## 📝 首次使用步骤

### 1. 启用立体声混音（重要！）

录制扬声器音频需要此设置：

1. 右键点击任务栏音量图标
2. 选择"声音设置"
3. 点击"更多声音设置"
4. 切换到"录制"标签
5. 右键空白处 → 勾选"显示已禁用的设备"
6. 找到"立体声混音"或"Stereo Mix"
7. 右键 → 启用

### 2. 生成配置文件

双击 `launcher.bat`，选择 `[1] 生成配置文件`

或在命令行运行：
```cmd
auto-recorder.exe gen-config
```

### 3. 测试录音

双击 `launcher.bat`，选择 `[3] 手动录音`

或在命令行运行：
```cmd
auto-recorder.exe record
```

说几句话，按 `Ctrl+C` 停止，录音文件会保存在 `recordings\` 文件夹。

### 4. 自动录音（可选）

如需监控微信、QQ 等通话应用：

双击 `launcher.bat`，选择 `[4] 自动录音`

或在命令行运行：
```cmd
auto-recorder.exe auto
```

## 🐛 常见问题

### Q: 双击 .exe 没反应
**A:** 这是命令行程序，请使用 `launcher.bat` 或命令提示符运行

### Q: 提示缺少 DLL
**A:** 安装 Visual C++ Redistributable：
- [下载链接](https://aka.ms/vs/17/release/vc_redist.x64.exe)

### Q: 杀毒软件报毒
**A:** 这是误报，程序完全开源。将文件夹添加到白名单

### Q: 录音文件很大
**A:** WAV 是无损格式。可以转换为 MP3：
```cmd
:: 使用 launcher.bat 选择 [5]
:: 或运行
convert_to_mp3.bat
```

### Q: 无法录制扬声器声音
**A:** 确保已启用"立体声混音"（见上文步骤）

### Q: 找不到通话应用
**A:** 编辑 `config.json`，添加应用的进程名：
```json
{
  "monitored_apps": [
    "WeChat.exe",
    "YourApp.exe"
  ]
}
```

## 📂 文件结构

```
auto-recorder/
├── auto-recorder.exe          主程序（命令行）
├── launcher.bat               图形启动器（推荐）
├── diagnose.bat              诊断工具
├── convert_to_mp3.bat        WAV转MP3工具
├── config.json               配置文件（首次运行生成）
├── recordings/               录音文件夹
│   ├── recording_20240207_153045.wav
│   └── ...
└── README_WINDOWS.md         本文档
```

## 🎯 快速命令参考

```cmd
:: 一键生成配置
auto-recorder.exe gen-config

:: 列出所有音频设备
auto-recorder.exe list-devices

:: 开始录音（Ctrl+C停止）
auto-recorder.exe record

:: 高质量录音
auto-recorder.exe record --sample-rate 48000

:: 自动录音模式（监控通话应用）
auto-recorder.exe auto

:: 查看详细日志
auto-recorder.exe --verbose auto
```

## 💡 提示

1. **推荐使用 `launcher.bat`**，最简单直观
2. **首次使用记得启用立体声混音**
3. **定期转换 WAV 为 MP3** 以节省空间
4. **遇到问题先运行 `diagnose.bat`**

## 📞 获取帮助

- 📖 查看完整文档：`USER_GUIDE_CN.md`
- 🔍 诊断问题：运行 `diagnose.bat`
- 💬 报告问题：GitHub Issues
- 📧 联系支持：见项目主页

---

**记住：直接双击 .exe 不会有任何显示，这是正常的！**

**请使用 `launcher.bat` 或命令提示符运行程序。**
