# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Planned
- Linux process monitoring support
- macOS process monitoring support
- GUI interface
- Hotkey control for recording
- Audio waveform visualization
- Support for multiple audio formats (WAV, FLAC, OGG)
- Cloud backup integration
- Recording file management interface

## [0.1.0] - 2024-02-07

### Added
- 🎤 Dual audio source recording (microphone + speaker)
- 🤖 Automatic call detection and recording (Windows only)
- 📝 Manual recording mode
- 🎵 Pure Rust MP3 encoding using `mp3lame`
- ⚙️ Configurable sample rate, bit rate, and quality
- 🔇 Silence detection with configurable threshold
- 💾 Automatic file saving with timestamps
- 📋 Process monitoring for common call apps:
  - WeChat
  - QQ
  - Feishu/Lark
  - Skype
  - Microsoft Teams
  - Zoom
  - Discord
  - DingTalk
- 🛠️ Command-line interface with multiple subcommands:
  - `auto` - Auto monitoring mode
  - `record` - Manual recording
  - `list-devices` - List audio devices
  - `gen-config` - Generate default config
- 📄 Comprehensive documentation:
  - User guide (English & Chinese)
  - Development documentation
  - Quick start guide
- 🚀 GitHub Actions for automated releases
- 🖥️ Multi-platform support:
  - Windows (x64, x86) - Full features
  - Linux (x64) - Manual recording only
  - macOS (Intel, ARM) - Manual recording only

### Technical Details
- Cross-platform audio I/O using `cpal`
- LAME MP3 encoder integration
- Windows API process monitoring
- Asynchronous runtime with `tokio`
- Structured logging with `tracing`
- Command-line parsing with `clap`
- JSON configuration with `serde`

### Known Limitations
- Automatic recording only works on Windows
- Requires "Stereo Mix" to be enabled on Windows
- Some sound cards may not support loopback recording
- No GUI interface in this version

[Unreleased]: https://github.com/yourusername/auto-recorder/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/yourusername/auto-recorder/releases/tag/v0.1.0
