# 重要更新说明

## WAV 格式录音

由于 `mp3lame` crate 在 crates.io 上不可用，项目已更新为使用 **WAV 格式**录音。

### 主要变化

1. **录音格式**: MP3 → WAV
   - 录音文件扩展名：`.mp3` → `.wav`
   - 编码库：`mp3lame` → `hound`

2. **优势**:
   - ✅ 无损音质
   - ✅ 更简单可靠
   - ✅ 无外部库依赖
   - ✅ 可直接在所有平台编译

3. **文件大小**:
   - WAV 文件比 MP3 大约 10 倍
   - 44100 Hz, 16-bit: 约 5 MB/分钟
   - 可使用提供的脚本转换为 MP3

### MP3 转换

如需 MP3 格式，使用提供的转换脚本：

#### Linux/macOS
```bash
# 安装 ffmpeg
sudo apt-get install ffmpeg  # Ubuntu/Debian
brew install ffmpeg          # macOS

# 转换
chmod +x convert_to_mp3.sh
./convert_to_mp3.sh
```

#### Windows
```cmd
# 1. 下载 ffmpeg: https://ffmpeg.org/download.html
# 2. 添加到 PATH
# 3. 运行转换脚本
convert_to_mp3.bat
```

### 配置变化

`config.json` 中的以下参数不再影响录音质量（仅用于 MP3 转换）：
- `bit_rate` - 保留用于将来
- `quality` - 保留用于将来

`sample_rate` 仍然有效，控制 WAV 文件的采样率。

### 文件大小对比

| 格式 | 1分钟 | 10分钟 | 60分钟 |
|------|-------|--------|--------|
| WAV (44.1kHz, 16bit) | ~5 MB | ~50 MB | ~300 MB |
| MP3 (128 kbps) | ~1 MB | ~10 MB | ~60 MB |
| MP3 (320 kbps) | ~2.4 MB | ~24 MB | ~144 MB |

### 推荐工作流

1. **使用 auto-recorder 录音**（生成 WAV 文件）
2. **定期转换为 MP3**（节省空间）
3. **删除原始 WAV 文件**（可选）

```bash
# 自动录音
auto-recorder auto

# 转换为 MP3
./convert_to_mp3.sh -b 192k -q 1

# 删除原始 WAV（谨慎）
rm recordings/*.wav
```

### 为什么使用 WAV 而不是 MP3？

1. **依赖问题**: `mp3lame` 不在 crates.io，需要系统安装 LAME 库
2. **可靠性**: WAV 格式更简单，100% 纯 Rust 实现
3. **跨平台**: WAV 在所有平台都能正常工作
4. **质量**: WAV 是无损格式，保留最高音质
5. **灵活性**: 可以后期选择任意压缩格式和参数

### 代码变化

- `src/mp3_encoder.rs` → 使用 `hound` 库
- `Mp3Encoder` → `AudioEncoder`
- 录音扩展名 `.mp3` → `.wav`

### 对用户的影响

**不影响功能**：
- ✅ 自动录音功能完全相同
- ✅ 手动录音功能完全相同
- ✅ 进程检测功能完全相同
- ✅ 所有配置参数仍然有效

**需要注意**：
- ⚠️ 文件更大（需要更多磁盘空间）
- ⚠️ 需要 MP3 时需手动转换
- ✅ 但音质更好（无损）

### 未来计划

考虑添加以下功能：
- [ ] 自动后台转换 WAV → MP3
- [ ] 内置 MP3 编码器（纯 Rust 实现）
- [ ] 支持多种格式选择（WAV/MP3/FLAC/OGG）
- [ ] 实时压缩选项

### 反馈

如有问题或建议，欢迎提交 Issue！
