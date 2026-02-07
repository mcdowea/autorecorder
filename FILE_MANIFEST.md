# 项目文件清单 📋

## 完整的文件结构

```
auto-recorder/
├── .github/
│   └── workflows/
│       └── release.yml          # GitHub Actions 自动发布工作流
│
├── src/
│   ├── main.rs                  # 程序入口，命令行解析
│   ├── config.rs                # 配置文件管理
│   ├── audio_capture.rs         # 音频捕获（cpal）
│   ├── mp3_encoder.rs           # MP3 编码（LAME）
│   ├── process_monitor.rs       # 进程监控（Windows API）
│   └── recorder.rs              # 录音核心逻辑
│
├── Cargo.toml                   # Rust 项目配置和依赖
├── .gitignore                   # Git 忽略文件配置
├── LICENSE                      # MIT 许可证
│
├── README.md                    # 项目主文档（英文）
├── QUICKSTART.md                # 快速开始指南
├── USER_GUIDE_CN.md             # 详细用户指南（中文）
├── DEVELOPMENT.md               # 开发者文档
│
├── config.example.json          # 配置文件示例
│
├── build.sh                     # Linux/macOS 构建脚本
└── build.bat                    # Windows 构建脚本
```

## 文件说明

### 核心源代码（src/）

#### main.rs (约 120 行)
- 程序入口点
- 命令行参数解析（使用 clap）
- 子命令路由（auto, record, list-devices, gen-config）
- 日志初始化
- Ctrl+C 信号处理

#### config.rs (约 70 行)
- `Config` 结构体定义
- 配置文件加载/保存
- JSON 序列化/反序列化
- 默认配置生成

#### audio_capture.rs (约 150 行)
- `AudioCapture` 结构体
- 音频设备枚举
- 麦克风和扬声器初始化
- 音频流创建
- 多种采样格式支持（f32, i16, u16）
- 多声道转单声道

#### mp3_encoder.rs (约 60 行)
- `Mp3Encoder` 结构体
- LAME 编码器配置
- f32 样本转 i16 PCM
- 实时 MP3 编码
- 缓冲区管理

#### process_monitor.rs (约 90 行)
- `ProcessMonitor` 结构体
- Windows 进程枚举（使用 ToolHelp API）
- 应用程序检测
- 跨平台桩实现

#### recorder.rs (约 180 行)
- `Recorder` 结构体
- 录音状态管理
- 自动监控循环
- 手动录音会话
- 音频混合（麦克风 + 扬声器）
- 静音检测
- 文件命名（时间戳）

### 配置和构建

#### Cargo.toml
- 项目元数据
- 依赖项列表：
  - cpal: 音频 I/O
  - mp3lame: MP3 编码
  - windows: Windows API
  - tokio: 异步运行时
  - clap: 命令行解析
  - serde/serde_json: 配置序列化
  - tracing: 日志
  - crossbeam-channel: 线程间通信
  - parking_lot: 同步原语
- 编译优化配置

#### .github/workflows/release.yml
- 多平台构建矩阵：
  - Windows (x64, x86)
  - Linux (x64)
  - macOS (Intel, ARM)
- 自动发布到 GitHub Releases
- 工件上传

### 文档

#### README.md
- 项目概述
- 功能特性
- 快速开始
- 使用方法
- 配置说明
- 技术栈
- 故障排除

#### QUICKSTART.md
- 5分钟快速上手
- 简化的安装步骤
- 基本命令示例
- 常用配置

#### USER_GUIDE_CN.md
- 详细的中文使用指南
- Windows 立体声混音配置
- 完整的配置说明
- 使用场景示例
- 常见问题解答
- 法律声明

#### DEVELOPMENT.md
- 项目架构说明
- 模块详解
- 音频流程图
- 自动录音流程图
- 开发指南
- 依赖说明
- 贡献指南
- 路线图

### 配置示例

#### config.example.json
- 完整的配置示例
- 包含所有可配置选项
- 常见应用列表
- 注释说明

### 构建脚本

#### build.sh (Linux/macOS)
- 检查 Rust 环境
- 运行 cargo check
- 编译 debug 和 release 版本
- 显示构建结果

#### build.bat (Windows)
- Windows 批处理版本
- 相同的构建流程
- 错误处理

## 代码统计

```
文件类型         文件数    代码行数
-------------------------------------
Rust 源码          6      ~670 行
配置文件           2       ~50 行
文档              5     ~1500 行
脚本              2       ~80 行
GitHub Actions    1      ~100 行
-------------------------------------
总计             16     ~2400 行
```

## 编译产物

### 构建后生成的文件

```
target/
├── debug/
│   ├── auto-recorder[.exe]        # Debug 版本（大，带调试符号）
│   └── ...
└── release/
    ├── auto-recorder[.exe]        # Release 版本（小，已优化）
    └── ...
```

### 运行时生成的文件

```
./
├── config.json                    # 用户配置（首次运行生成）
└── recordings/                    # 录音输出目录
    ├── recording_20240207_153045.mp3
    ├── recording_20240207_160230.mp3
    └── ...
```

## 依赖关系图

```
main.rs
  ├─→ config.rs
  │     └─→ serde, serde_json
  │
  ├─→ recorder.rs
  │     ├─→ audio_capture.rs
  │     │     └─→ cpal, crossbeam-channel
  │     │
  │     ├─→ mp3_encoder.rs
  │     │     └─→ mp3lame
  │     │
  │     └─→ process_monitor.rs
  │           └─→ windows (仅 Windows)
  │
  └─→ clap, tracing
```

## 下一步

1. **开始开发**：阅读 [DEVELOPMENT.md](DEVELOPMENT.md)
2. **了解使用**：阅读 [USER_GUIDE_CN.md](USER_GUIDE_CN.md)
3. **快速测试**：阅读 [QUICKSTART.md](QUICKSTART.md)
4. **查看代码**：从 `src/main.rs` 开始
