# 编译选项说明

## 🎯 两种编译模式

本项目支持两种编译模式：

### 1. 无 GUI 模式（推荐，编译快）

```bash
# 编译无 GUI 版本（默认）
cargo build --release --no-default-features

# 或指定目标平台
cargo build --release --no-default-features --target x86_64-pc-windows-msvc
```

**优点**:
- ✅ 编译速度快（少了 GUI 依赖）
- ✅ 体积更小
- ✅ 避免 GUI 库的兼容性问题
- ✅ 适合服务器和后台运行

**使用方法**:
```bash
# 自动录音模式
auto-audio-recorder.exe run

# 手动录音
auto-audio-recorder.exe start

# 查看配置
auto-audio-recorder.exe config
```

---

### 2. 完整 GUI 模式

```bash
# 编译带 GUI 的版本
cargo build --release --features gui

# 或指定目标平台
cargo build --release --features gui --target x86_64-pc-windows-msvc
```

**优点**:
- ✅ 图形界面操作
- ✅ 实时查看状态
- ✅ 方便调整设置

**使用方法**:
```bash
# 启动 GUI
auto-audio-recorder.exe gui

# 或直接运行
auto-audio-recorder.exe
```

---

## 📝 GitHub Actions 配置

如果使用 GitHub Actions 自动构建，修改 `.github/workflows/release.yml`:

### 无 GUI 版本（推荐）

```yaml
- name: Build
  run: cargo build --release --no-default-features --target ${{ matrix.platform.target }}
```

### 带 GUI 版本

```yaml
- name: Build
  run: cargo build --release --features gui --target ${{ matrix.platform.target }}
```

---

## 🔧 故障排查

### GUI 编译错误

如果遇到类似错误：
```
error[E0432]: unresolved import `winapi::um::winuser`
```

**解决方案 1**: 使用无 GUI 模式编译
```bash
cargo build --release --no-default-features
```

**解决方案 2**: 更新依赖
```bash
cargo update
cargo build --release --features gui
```

**解决方案 3**: 清理并重新编译
```bash
cargo clean
cargo build --release --no-default-features
```

---

## 💡 推荐配置

### 开发环境
```bash
# 快速编译和测试
cargo build --no-default-features
cargo run --no-default-features -- run
```

### 生产环境
```bash
# 优化编译
cargo build --release --no-default-features --target x86_64-pc-windows-msvc
```

### 分发版本
如果需要提供两个版本：

1. **命令行版本** (体积小，速度快)
   ```bash
   cargo build --release --no-default-features
   ```
   
2. **GUI 版本** (用户友好)
   ```bash
   cargo build --release --features gui
   ```

---

## 📦 功能对比

| 功能 | 无 GUI | 带 GUI |
|------|--------|--------|
| 自动录音 | ✅ | ✅ |
| 手动录音 | ✅ | ✅ |
| 进程监控 | ✅ | ✅ |
| MP3 编码 | ✅ | ✅ |
| 配置文件 | ✅ | ✅ |
| 命令行 | ✅ | ✅ |
| 图形界面 | ❌ | ✅ |
| 编译速度 | 快 | 慢 |
| 文件大小 | 小 | 大 |
| 依赖数量 | 少 | 多 |

---

## 🚀 快速开始

### 最快编译方式

```bash
# 1. 克隆项目
git clone https://github.com/yourusername/auto-audio-recorder.git
cd auto-audio-recorder

# 2. 无 GUI 编译（最快）
cargo build --release --no-default-features

# 3. 运行
.\target\release\auto-audio-recorder.exe run
```

### 完整功能编译

```bash
# 1. 编译带 GUI
cargo build --release --features gui

# 2. 运行 GUI
.\target\release\auto-audio-recorder.exe gui
```

---

## 🔍 检查编译配置

查看当前编译配置：

```bash
# 查看启用的 features
cargo tree -e features

# 只编译不运行
cargo build --release --no-default-features

# 查看编译大小
dir target\release\auto-audio-recorder.exe
```

---

**推荐**: 如果只需要核心录音功能，使用 `--no-default-features` 编译，可以避免 GUI 相关的所有问题。
