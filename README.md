# transcriber

A fast video transcription CLI tool powered by whisper.cpp. Extracts audio from video files and transcribes it to text with GPU acceleration.

## Features

- **Video → Audio → Text**: Automatic audio extraction via FFmpeg
- **GPU Accelerated**: Metal (macOS M*), Vulkan (NVIDIA), CUDA, or CPU fallback
- **Multiple Output Formats**: TXT, SRT (subtitle), JSON (with word-level timestamps)
- **Batch Processing**: Transcribe entire directories recursively
- **Model Quantization**: Q4_K/Q5_K/Q6_K/Q8_0 for speed/size tradeoffs
- **Configurable**: YAML configuration file with CLI argument overrides
- **Progress Reporting**: Real-time progress bars for downloads and transcription

## Installation

### Prerequisites

- **FFmpeg**: Required for audio extraction
  - macOS: `brew install ffmpeg`
  - Ubuntu: `sudo apt install ffmpeg`
  - Windows: Download from https://ffmpeg.org/

### Homebrew (macOS / Linux)

```bash
brew tap RedAtman/tap
brew install transcriber
```

> Taps into [RedAtman/homebrew-tap](https://github.com/RedAtman/homebrew-tap). Formula is automatically updated on each release.

### From Source

```bash
git clone <repo-url>
cd transcriber
cargo build --release
```

The binary is at `./target/release/transcriber`.

## Usage

### Single File Transcription

```bash
# Transcribe with default settings (base model, txt output)
transcriber -i video.mp4

# Specify model and language
transcriber -i video.mp4 -m medium -l zh

# Custom output directory and SRT format
transcriber -i video.mp4 -o ./subtitles --format srt
```

### Batch Processing

```bash
# Transcribe all videos in a directory
transcriber -d ./videos

# Skip already-transcribed files
transcriber -d ./videos --skip-existing

# Multiple output formats
transcriber -d ./videos --format "txt,srt,json"
```

### Inference Parameters

```bash
# Initial prompt for decoder context
transcriber -i video.mp4 --initial-prompt "technical terms"

# Sampling temperature (0.0 = deterministic, 1.0 = more random)
transcriber -i video.mp4 --temperature 0.2

# Suppress non-speech tokens
transcriber -i video.mp4 --suppress-non-speech

# No-speech detection threshold
transcriber -i video.mp4 --no-speech-threshold 0.5

# Split on word boundaries
transcriber -i video.mp4 --split-on-word

# Combined example
transcriber -i video.mp4 -m medium -l en --initial-prompt "technology" --temperature 0.3
transcriber -i video.mp4 -m medium -l zh --initial-prompt "科技" --temperature 0.3
```

### Configuration

```bash
# Generate default config file
transcriber init

# Use custom config
transcriber -i video.mp4 --config ./my-config.yaml
```

Default config location: `~/.config/transcriber/config.yaml`

### Available Models

| Model | Size | Description |
|-------|------|-------------|
| tiny | 75 MB | Fastest, lowest quality |
| base | 148 MB | Default, balanced |
| small | 488 MB | Better quality |
| medium | 1.5 GB | High quality |
| large-v3-turbo | 800 MB | Best quality/speed ratio |

## Output Formats

- **TXT**: Plain text, one segment per line
- **SRT**: SubRip subtitle format with timestamps
- **JSON**: Structured data with word-level timing and metadata

## License

MIT
