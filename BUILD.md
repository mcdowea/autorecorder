# 构建说明

## 环境准备

### Windows

```powershell
# 安装 Rust
# 访问 https://rustup.rs/ 下载并安装

# 验证安装
rustc --version
cargo --version
```

### macOS

```bash
# 安装 Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 验证安装
rustc --version
cargo --version
```

### Linux (Ubuntu/Debian)

```bash
# 安装 Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 安装依赖
sudo apt-get update
sudo apt-get install -y \
    libasound2-dev \
    pkg-config \
    build-essential

# 验证安装
rustc --version
cargo --version
```

## 编译项目

### 开发模式

```bash
# 克隆仓库
git clone https://github.com/yourusername/auto-audio-recorder.git
cd auto-audio-recorder

# 开发构建（包含调试信息，编译快）
cargo build

# 运行程序
cargo run

# 运行并查看日志
RUST_LOG=debug cargo run
```

### 发布模式

```bash
# 发布构建（优化编译，体积小，运行快）
cargo build --release

# 可执行文件位于
# Windows: target/release/auto-audio-recorder.exe
# macOS/Linux: target/release/auto-audio-recorder
```

### 交叉编译

#### Windows 上编译其他平台

```powershell
# 安装目标平台工具链
rustup target add x86_64-unknown-linux-gnu
rustup target add x86_64-apple-darwin

# 编译 Linux 版本（需要额外工具）
cargo build --release --target x86_64-unknown-linux-gnu
```

#### Linux 上编译 Windows 版本

```bash
# 安装 MinGW
sudo apt-get install mingw-w64

# 安装目标
rustup target add x86_64-pc-windows-gnu

# 编译
cargo build --release --target x86_64-pc-windows-gnu
```

## 运行测试

```bash
# 运行所有测试
cargo test

# 运行特定模块测试
cargo test audio_capture

# 显示测试输出
cargo test -- --nocapture
```

## 性能分析

```bash
# 安装 flamegraph
cargo install flamegraph

# 生成火焰图
cargo flamegraph

# 使用 perf（Linux）
perf record -g cargo run --release
perf report
```

## 代码检查

```bash
# 运行 clippy（代码检查工具）
cargo clippy

# 格式化代码
cargo fmt

# 检查代码格式
cargo fmt -- --check
```

## 清理构建

```bash
# 清理所有构建产物
cargo clean

# 清理并重新构建
cargo clean && cargo build --release
```

## 依赖更新

```bash
# 检查过时的依赖
cargo outdated

# 更新依赖
cargo update

# 更新到最新兼容版本
cargo update --aggressive
```

## 常见问题

### Windows 编译错误

如果遇到链接错误，确保安装了 Visual Studio Build Tools：
- 访问 https://visualstudio.microsoft.com/downloads/
- 下载并安装 "Build Tools for Visual Studio"
- 选择 "C++ build tools" 工作负载

### Linux 音频库缺失

```bash
# Ubuntu/Debian
sudo apt-get install libasound2-dev

# Fedora/RHEL
sudo dnf install alsa-lib-devel

# Arch Linux
sudo pacman -S alsa-lib
```

### macOS 权限问题

运行时可能需要授予麦克风权限：
1. 系统偏好设置 → 安全性与隐私 → 隐私 → 麦克风
2. 勾选终端或您的应用程序

## 优化建议

### 减小二进制大小

在 `Cargo.toml` 中添加：

```toml
[profile.release]
opt-level = "z"  # 优化体积
lto = true       # 链接时优化
codegen-units = 1
strip = true     # 移除符号
```

### 加快编译速度

```bash
# 使用 sccache 缓存编译结果
cargo install sccache
export RUSTC_WRAPPER=sccache

# 或使用 mold 链接器（Linux）
sudo apt-get install mold
export RUSTFLAGS="-C link-arg=-fuse-ld=mold"
```

## 打包发布

### 创建发布包

```bash
# 编译发布版本
cargo build --release

# 创建压缩包（Linux/macOS）
cd target/release
tar -czf auto-audio-recorder-linux-x64.tar.gz auto-audio-recorder
zip auto-audio-recorder-linux-x64.zip auto-audio-recorder

# Windows
# 使用 7-Zip 或 WinRAR 创建压缩包
```

### 使用 GitHub Actions

项目已配置 GitHub Actions 自动构建：
1. 推送代码到 GitHub
2. 创建标签触发构建：
   ```bash
   git tag v0.1.0
   git push origin v0.1.0
   ```
3. GitHub Actions 会自动构建并创建 Release

## 技术支持

如有编译问题，请：
1. 查看 [README.md](README.md)
2. 提交 [Issue](https://github.com/yourusername/auto-audio-recorder/issues)
3. 访问 [Rust 官方论坛](https://users.rust-lang.org/)
