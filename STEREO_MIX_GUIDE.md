# 立体声混音设置指南

## ⚠️ 重要提示

如果你看到以下警告信息：
```
WARN: No loopback device found. Please enable 'Stereo Mix' in Windows sound settings.
WARN: Speaker recording will be disabled. Only microphone will be recorded.
```

这意味着程序无法录制扬声器（电脑播放）的声音，只能录制麦克风。

要同时录制扬声器和麦克风，你需要启用 **立体声混音** 设备。

## 🔧 如何启用立体声混音

### Windows 10/11 详细步骤

#### 方法 1：通过设置应用

1. **打开声音设置**
   - 右键点击任务栏右下角的音量图标 🔊
   - 点击 "声音设置" 或 "打开声音设置"

2. **进入高级声音设置**
   - 向下滚动找到 "高级" 或 "更多声音设置"
   - 点击 "声音控制面板" 或类似选项

3. **显示已禁用的设备**
   - 在打开的窗口中，切换到 **"录制"** 标签页
   - 在空白处 **右键点击**
   - 确保勾选以下两项：
     - ✓ **显示已禁用的设备**
     - ✓ **显示已断开的设备**

4. **启用立体声混音**
   - 现在你应该能看到 "立体声混音" 设备
   - 常见名称：
     - 🇨🇳 立体声混音
     - 🇺🇸 Stereo Mix
     - 🇺🇸 What U Hear (Creative 声卡)
     - 🇺🇸 Wave Out Mix
   - **右键点击** 该设备
   - 选择 **"启用"**

5. **（可选）设为默认录音设备**
   - 右键点击已启用的 "立体声混音"
   - 选择 "设为默认设备" 或 "设为默认通信设备"

#### 方法 2：通过运行命令

1. 按 `Win + R` 打开运行对话框
2. 输入 `mmsys.cpl` 并按回车
3. 按照上面方法 1 的步骤 3-5 操作

### 验证设置

启用后，重新运行程序：
```cmd
.\auto-recorder.exe record
```

你应该看到：
```
INFO: Speaker loopback device found: 立体声混音 (Realtek High Definition Audio)
```

而不是之前的警告信息。

## 🆘 找不到立体声混音？

### 可能原因 1：驱动程序太旧

**解决方法：**
1. 更新声卡驱动程序
2. 访问声卡制造商网站：
   - Realtek: https://www.realtek.com/
   - Creative: https://support.creative.com/
   - Intel: https://www.intel.com/
3. 或使用 Windows 更新自动更新驱动

### 可能原因 2：某些声卡不支持

**解决方法：使用虚拟音频设备**

#### 选项 A：VB-Audio Virtual Cable（免费）

1. 下载：https://vb-audio.com/Cable/
2. 安装后会创建虚拟音频设备
3. 在 Windows 声音设置中：
   - 将默认播放设备设为 "CABLE Input"
   - 将默认录音设备设为 "CABLE Output"
4. 重新运行程序

#### 选项 B：VoiceMeeter（免费，功能更强）

1. 下载：https://vb-audio.com/Voicemeeter/
2. 安装并配置音频路由
3. 提供更多控制选项

### 可能原因 3：笔记本电脑限制

某些笔记本电脑（特别是低端型号）可能完全不支持立体声混音功能。

**解决方法：**
- 使用上述虚拟音频设备
- 或接受仅录制麦克风

## 📊 功能对比

| 场景 | 立体声混音已启用 | 立体声混音未启用 |
|------|-----------------|-----------------|
| 麦克风声音 | ✅ 录制 | ✅ 录制 |
| 扬声器声音 | ✅ 录制 | ❌ 不录制 |
| 微信/QQ 语音对方声音 | ✅ 录制 | ❌ 不录制 |
| 系统声音/音乐 | ✅ 录制 | ❌ 不录制 |
| 通话录音完整性 | ✅ 双向完整 | ⚠️ 仅自己说话 |

## 💡 使用建议

### 如果你只需要录制自己说话
- 无需启用立体声混音
- 直接使用即可

### 如果你需要录制通话对话
- **必须**启用立体声混音
- 否则只能录到自己的声音

### 如果你需要录制电脑播放的音乐/视频
- **必须**启用立体声混音

## 🎯 快速测试

启用立体声混音后，可以这样测试：

1. 运行录音程序
   ```cmd
   .\auto-recorder.exe record
   ```

2. 在电脑上播放一首歌或视频

3. 对着麦克风说话

4. 按 Ctrl+C 停止录音

5. 播放录音文件，应该能听到：
   - ✅ 你的声音（来自麦克风）
   - ✅ 音乐/视频声音（来自扬声器）

## 🔍 故障排除

### Q: 启用了立体声混音，但程序仍然提示找不到

**A:** 尝试以下步骤：
1. 重启程序
2. 检查设备名称是否是标准名称
3. 查看设备管理器中的声卡型号
4. 运行 `auto-recorder.exe list-devices` 查看所有设备

### Q: 启用后声音变小或有回音

**A:** 调整立体声混音的音量：
1. 在录制设备中右键 "立体声混音"
2. 选择 "属性"
3. 在 "级别" 标签页调整音量
4. 推荐设置为 50-70%

### Q: 能听到自己的声音（回音）

**A:** 关闭立体声混音的监听：
1. 右键 "立体声混音" → 属性
2. 切换到 "侦听" 标签页
3. **取消勾选** "侦听此设备"

## 📞 获取帮助

如果以上方法都不能解决问题：

1. 运行诊断工具
   ```cmd
   .\diagnose.bat
   ```

2. 截图错误信息

3. 在 GitHub 提交 Issue：
   https://github.com/yourusername/auto-recorder/issues

4. 提供以下信息：
   - Windows 版本
   - 声卡型号
   - 错误信息截图
   - `list-devices` 命令的输出

## ✅ 成功案例

如果设置正确，你会看到：

```
INFO: Auto Recorder v0.2.0
INFO: ==================
INFO: Manual recording mode
INFO: Microphone initialized: 麦克风 (Realtek High Definition Audio)
INFO: Speaker loopback device found: 立体声混音 (Realtek High Definition Audio)
INFO: Speaker recording enabled
INFO: Recording started...
```

注意 "Speaker recording enabled" 这一行！

---

**记住：启用立体声混音是录制完整通话对话的关键！** 🎙️
