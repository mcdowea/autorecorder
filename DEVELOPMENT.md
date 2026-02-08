# 开发文档

## 项目结构

```
auto-recorder/
├── src/
│   ├── main.rs              # 程序入口
│   ├── config.rs            # 配置管理
│   ├── audio_capture.rs     # 音频捕获
│   ├── mp3_encoder.rs       # MP3 编码
│   ├── process_monitor.rs   # 进程监控
│   └── recorder.rs          # 录音核心逻辑
├── .github/
│   └── workflows/
│       └── release.yml      # GitHub Actions 自动发布
├── Cargo.toml
├── README.md
└── LICENSE
```

## 核心模块说明

### 1. audio_capture.rs

负责音频设备的初始化和音频流的创建。

**主要功能：**
- 列出可用的音频设备
- 初始化麦克风和扬声器设备
- 创建音频流并捕获样本
- 自动处理不同的采样格式 (f32, i16, u16)
- 多声道转单声道混音

**关键方法：**
```rust
AudioCapture::new()           // 创建捕获器
init_microphone()             // 初始化麦克风
init_speaker()                // 初始化扬声器
create_stream()               // 创建音频流
```

### 2. mp3_encoder.rs

使用 LAME 库进行 MP3 编码。

**主要功能：**
- 配置 MP3 编码参数（采样率、比特率、质量）
- 将 f32 音频样本转换为 i16 PCM
- 实时编码并写入文件
- 完成编码并刷新缓冲区

**关键方法：**
```rust
Mp3Encoder::new()             // 创建编码器
encode_samples()              // 编码音频样本
finish()                      // 完成编码
```

### 3. process_monitor.rs

监控系统进程，检测通话应用。

**平台支持：**
- ✅ Windows: 使用 Windows API 完整实现
- ⚠️ Linux/macOS: 桩实现（总是返回 false）

**主要功能：**
- 获取运行中的进程列表
- 检查指定的应用是否在运行
- 用于自动录音模式的触发

**关键方法：**
```rust
ProcessMonitor::new()         // 创建监控器
is_call_active()              // 检查通话是否活跃
```

### 4. recorder.rs

录音核心逻辑，协调各模块工作。

**主要功能：**
- 管理录音状态
- 混合麦克风和扬声器音频
- 静音检测
- 自动模式下的通话检测循环
- 生成带时间戳的文件名

**关键方法：**
```rust
Recorder::new()                    // 创建录音器
start_manual_recording()           // 手动录音
start_auto_monitoring()            // 自动监控模式
record_session()                   // 执行录音会话
```

### 5. config.rs

配置文件管理。

**主要功能：**
- 定义配置结构
- 从 JSON 加载配置
- 保存配置到文件
- 提供默认配置

## 音频流程

```
┌─────────────┐     ┌─────────────┐
│ Microphone  │────▶│   cpal      │
└─────────────┘     │  Capture    │
                    └──────┬──────┘
                           │ f32 samples
                           ▼
                    ┌──────────────┐
                    │    Mixer     │
                    │  (Average)   │
                    └──────┬───────┘
┌─────────────┐           │
│  Speaker    │────▶      │
│ (Loopback)  │           │ Mixed f32
└─────────────┘           ▼
                    ┌──────────────┐
                    │ Silence      │
                    │ Detection    │
                    └──────┬───────┘
                           │
                           ▼
                    ┌──────────────┐
                    │ MP3 Encoder  │
                    │  (LAME)      │
                    └──────┬───────┘
                           │
                           ▼
                    ┌──────────────┐
                    │  MP3 File    │
                    └──────────────┘
```

## 自动录音流程

```
┌─────────────────────────┐
│  Start Monitoring       │
└────────────┬────────────┘
             │
             ▼
      ┌─────────────┐
      │ Check Apps  │◀─────┐
      └──────┬──────┘      │
             │              │
         ┌───▼───┐          │
         │ Call? │          │
         └───┬───┘          │
             │              │
      No ◀───┴───▶ Yes      │
      │              │      │
      │         ┌────▼────┐ │
      │         │ Record  │ │
      │         └────┬────┘ │
      │              │      │
      │         ┌────▼────┐ │
      │         │  Call   │ │
      │         │ Ended?  │ │
      │         └────┬────┘ │
      │              │      │
      │         Yes  │      │
      │         ┌────▼────┐ │
      │         │  Stop   │ │
      │         │ & Save  │ │
      │         └────┬────┘ │
      │              │      │
      └──────────────┴──────┘
             Sleep 2s
```

## 开发指南

### 本地开发

```bash
# 检查代码
cargo check

# 运行测试
cargo test

# 运行（开发模式）
cargo run -- list-devices
cargo run -- gen-config
cargo run -- --verbose record

# 构建发布版本
cargo build --release
```

### 添加新的监控应用

编辑 `config.json`：

```json
{
  "monitored_apps": [
    "YourApp.exe",
    // ... 其他应用
  ]
}
```

### 调整音频质量

**高质量设置：**
```json
{
  "sample_rate": 48000,
  "bit_rate": 320,
  "quality": 0
}
```

**平衡设置（推荐）：**
```json
{
  "sample_rate": 44100,
  "bit_rate": 128,
  "quality": 2
}
```

**低质量/小文件：**
```json
{
  "sample_rate": 22050,
  "bit_rate": 64,
  "quality": 7
}
```

## 依赖说明

### 核心依赖

- **cpal**: 跨平台音频 I/O，支持 Windows (WASAPI), Linux (ALSA/PulseAudio), macOS (CoreAudio)
- **mp3lame**: LAME MP3 编码器的 Rust 绑定
- **windows**: Windows API 访问（仅 Windows）

### 工具依赖

- **tokio**: 异步运行时（虽然当前主要用同步代码）
- **clap**: 命令行参数解析
- **serde**: 序列化/反序列化
- **tracing**: 结构化日志

## 常见问题

### Q: 为什么 Linux/macOS 不支持自动录音？

A: 进程监控实现使用了 Windows API。Linux/macOS 需要不同的实现方式（如 `/proc` 或 `ps` 命令），未来版本可能会添加。

### Q: 如何提高录音质量？

A: 
1. 提高 `sample_rate` (48000)
2. 提高 `bit_rate` (192-320)
3. 降低 `quality` (0-1)

### Q: 扬声器录音没有声音？

A: Windows 用户需要启用"立体声混音"设备。某些声卡可能不支持此功能。

### Q: 如何减小文件大小？

A:
1. 降低 `sample_rate` (22050)
2. 降低 `bit_rate` (64-96)
3. 提高 `quality` (7-9)

## 贡献指南

1. Fork 项目
2. 创建特性分支 (`git checkout -b feature/AmazingFeature`)
3. 提交更改 (`git commit -m 'Add some AmazingFeature'`)
4. 推送到分支 (`git push origin feature/AmazingFeature`)
5. 开启 Pull Request

## 路线图

- [ ] Linux 进程监控支持
- [ ] macOS 进程监控支持
- [ ] GUI 界面
- [ ] 热键控制录音
- [ ] 音频波形可视化
- [ ] 支持多种音频格式（WAV, FLAC, OGG）
- [ ] 云端自动备份
- [ ] 录音文件管理界面
