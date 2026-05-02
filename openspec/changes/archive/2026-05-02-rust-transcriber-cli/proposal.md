## Why

将 Python 版本的视频转录工具 (`douyin/cli/transcribe.py`) 重构为独立的 Rust CLI 工具，提供跨平台高性能的音视频转文本功能。当前 Python 实现依赖 `faster-whisper`，在 macOS/Linux 多平台上的 GPU 加速和性能优化存在局限性。

## What Changes

- 创建全新的 Rust CLI 工具 `transcriber`
- 基于 `whisper-rs` (whisper.cpp 的 Rust 绑定) 实现转录功能
- 支持多平台 GPU 加速 (Metal/CUDA/Vulkan) 和 CPU fallback
- YAML 配置文件支持自定义所有参数
- 支持单文件和批量目录转录
- 多格式输出 (txt/srt/json)

## Capabilities

### New Capabilities

- `video-transcription`: 核心转录功能，支持视频→音频→文本的全流程
- `configuration-management`: YAML 配置文件加载、校验、CLI 参数覆盖
- `model-management`: 模型自动下载、本地缓存、量化支持
- `audio-extraction`: FFmpeg 音频提取（自动检测/bundler fallback）
- `output-formats`: 多格式输出 (txt/srt/json)

## Impact

- 新增 Rust 项目: `Cargo.toml`, `src/`
- 新增配置文件: `~/.config/transcriber/config.yaml`
- 新增缓存目录: `~/.cache/transcriber/`
- 依赖: `whisper-rs`, `clap`, `anyhow`, `indicatif`, `serde_yaml`