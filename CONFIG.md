# 配置文件说明

## 配置文件位置

配置文件 `smart_recorder_config.json` 会在首次运行程序后自动生成在程序所在目录。

## 配置选项详解

```json
{
  "output_path": "D:\\Recordings",           // 录音文件保存路径
  "auto_create_folders": true,               // 是否按应用名自动创建文件夹
  "sample_rate": 48000,                      // 采样率 (Hz)
  "bit_rate": 128,                           // 比特率 (kbps, 仅MP3)
  "audio_format": "mp3",                     // 音频格式: "mp3" 或 "wav"
  "mp3_quality": 2,                          // MP3编码质量 (0-9)
  "mic_gain": 1.0,                           // 麦克风增益 (0.0-2.0)
  "speaker_gain": 1.0,                       // 扬声器增益 (0.0-2.0)
  "blacklist": "chrome.exe,firefox.exe",     // 进程黑名单 (逗号分隔)
  "auto_start": false,                       // 开机自动启动
  "minimize_to_tray": true,                  // 最小化到托盘
  "min_duration_seconds": 3                  // 最小录音时长(秒)
}
```

## 采样率 (sample_rate)

采样率越高,录音质量越好,但文件也越大。

| 采样率 | 适用场景 | 文件大小 |
|--------|----------|----------|
| 16000 Hz | 语音通话 (低质量) | 最小 |
| 22050 Hz | 一般语音录音 | 较小 |
| 44100 Hz | CD音质,音乐通话 | 中等 |
| 48000 Hz | 高质量录音 (推荐) | 较大 |
| 96000 Hz | 专业录音 | 最大 |

## 比特率 (bit_rate)

仅适用于 MP3 格式,比特率越高音质越好。

| 比特率 | 质量等级 | 适用场景 |
|--------|----------|----------|
| 64 kbps | 低质量 | 仅语音,极小文件 |
| 96 kbps | 一般 | 语音通话 |
| 128 kbps | 标准 | 日常录音 (推荐) |
| 192 kbps | 高质量 | 重要会议 |
| 256 kbps | 极高质量 | 音乐或专业用途 |
| 320 kbps | 最高质量 | 专业录音 |

## MP3 编码质量 (mp3_quality)

控制 LAME 编码器的质量/速度平衡。

| 值 | 说明 | 推荐使用场景 |
|----|------|--------------|
| 0 | 最高质量,最慢 | 专业录音,不在意编码时间 |
| 2 | 高质量 (推荐) | 日常使用的最佳平衡 |
| 5 | 标准质量 | 快速编码,质量尚可 |
| 7 | 低质量,快速 | 仅需要录制语音 |
| 9 | 最低质量,最快 | 测试或临时录音 |

## 音量增益 (mic_gain / speaker_gain)

- 范围: 0.0 - 2.0
- 1.0 = 原始音量
- < 1.0 = 降低音量
- > 1.0 = 增大音量

**注意**: 增益过大可能导致音频失真。

## 进程黑名单 (blacklist)

添加不希望触发录音的程序,多个程序用逗号分隔。

常见黑名单程序:
- `chrome.exe` - Google Chrome 浏览器
- `firefox.exe` - Firefox 浏览器
- `msedge.exe` - Microsoft Edge 浏览器
- `explorer.exe` - Windows 资源管理器
- `Discord.exe` - Discord (如果不想录制游戏语音)

## 最小录音时长 (min_duration_seconds)

设置最小录音时长(秒),低于此时长的录音会被自动丢弃。

- 建议值: 3-5 秒
- 用途: 避免误触发或极短的麦克风使用被录制

## 文件格式选择

### MP3 格式
- ✅ 文件小,易于分享
- ✅ 兼容性好
- ❌ 有损压缩
- 适合: 日常录音,语音通话

### WAV 格式
- ✅ 无损音质
- ✅ 适合后期处理
- ❌ 文件较大
- 适合: 专业录音,需要编辑的场景

## 推荐配置

### 日常通话录音
```json
{
  "sample_rate": 48000,
  "bit_rate": 128,
  "audio_format": "mp3",
  "mp3_quality": 2
}
```

### 高质量会议录音
```json
{
  "sample_rate": 48000,
  "bit_rate": 192,
  "audio_format": "mp3",
  "mp3_quality": 2
}
```

### 专业录音 (无损)
```json
{
  "sample_rate": 96000,
  "audio_format": "wav"
}
```

### 节省空间 (语音)
```json
{
  "sample_rate": 22050,
  "bit_rate": 96,
  "audio_format": "mp3",
  "mp3_quality": 5
}
```
