# transcriber

A fast video transcription CLI powered by [whisper.cpp](https://github.com/ggerganov/whisper.cpp).
Point it at a video file or a folder: it extracts the audio with FFmpeg, transcribes it
locally with GPU acceleration (Metal / CUDA / Vulkan) or on the CPU, and writes
TXT, SRT, JSON, or Markdown.

Audio never leaves the machine. The only network access is the one-time model download.

## Quick start

```bash
brew install ffmpeg                              # required at runtime
brew tap RedAtman/tap && brew install transcriber

transcriber -i video.mp4                         # → video.transcript.txt
transcriber -d ./recordings -l zh --format srt   # whole folder, Chinese, subtitles
```

## Installation

### 1. FFmpeg (required)

`ffmpeg` and `ffprobe` must be on `PATH`; the tool refuses to start without them.

| Platform | Command |
|----------|---------|
| macOS | `brew install ffmpeg` |
| Debian / Ubuntu | `sudo apt install ffmpeg` |
| Windows | download from <https://ffmpeg.org/download.html> and add it to `PATH` |

### 2. transcriber

**Homebrew (macOS, Linux):**

```bash
brew tap RedAtman/tap
brew install transcriber
```

**Prebuilt binary** — grab the asset for your platform from
[Releases](https://github.com/RedAtman/transcriber/releases/latest) and put it on `PATH`:

| Platform | Asset |
|----------|-------|
| macOS Apple Silicon | `transcriber-macos-aarch64` |
| macOS Intel | `transcriber-macos-x86_64` |
| Linux x86_64 | `transcriber-linux-x86_64` |
| Windows x86_64 | `transcriber-windows-x86_64.exe` |

**From source:**

```bash
git clone https://github.com/RedAtman/transcriber.git
cd transcriber
cargo build --release       # binary at ./target/release/transcriber
```

Building compiles whisper.cpp, so it needs a C/C++ toolchain plus these system
libraries — the same set the release build installs:

| Platform | Packages |
|----------|----------|
| macOS | `brew install cmake pkg-config opus libsndfile libsamplerate` |
| Debian / Ubuntu | `sudo apt install build-essential cmake pkg-config libopus-dev libasound2-dev libsamplerate0-dev libssl-dev` |

## Usage

### Transcribe one file

```bash
transcriber -i video.mp4                       # default model (base), auto language, txt
transcriber -i video.mp4 -m medium -l zh       # bigger model, Chinese
transcriber -i video.mp4 -o ./subtitles --format srt
```

### Transcribe a folder

`-d` recurses into subfolders and picks up `mp4`, `mov`, `avi`, `mkv`, `webm`, `m4v`
(case-insensitive). `-d` and `-i` are mutually exclusive.

```bash
transcriber -d ./videos
transcriber -d ./videos --format txt,srt,json
transcriber -d ./videos --format all           # txt + srt + json + md
```

Files that already have output are skipped by default (`output.skip_existing: true`
in the config). Delete the existing `.transcript.*` files, or set
`output.skip_existing: false` in the config file, to re-transcribe them — there is no
CLI flag for turning it off.

### Where the output goes

Each format is written to `<stem>.transcript.<ext>`, where `<stem>` is the input
filename without its extension:

```
recordings/2026-05-01-talk.mkv
  → recordings/2026-05-01-talk.transcript.txt
  → recordings/2026-05-01-talk.transcript.srt
```

By default they land next to the input file. `--output DIR` (or `output.directory` in
the config) writes them to `DIR` instead and creates it if needed.

### Custom metadata

`--meta` is repeatable and lands in the Markdown YAML front matter (and in the JSON
output). Quote values that contain spaces.

```bash
transcriber -i talk.mp4 --format md \
  --meta title="Quarterly review" --meta speaker="Ada" --meta location=Beijing
```

### Tuning the decoder

```bash
transcriber -i talk.mp4 --initial-prompt "Kubernetes, gRPC, observability"
transcriber -i talk.mp4 --temperature 0.2          # 0.0 = deterministic (default)
transcriber -i talk.mp4 --suppress-non-speech      # drop coughs / background noise
transcriber -i talk.mp4 --no-speech-threshold 0.4  # more aggressive silence detection
transcriber -i talk.mp4 --max-segment-length 42 --split-on-word
```

## CLI reference

| Flag | Default | Notes |
|------|---------|-------|
| `-i, --input FILE` | – | Single input file. Conflicts with `-d`. |
| `-d, --dir DIR` | – | Recurse a folder. Conflicts with `-i`. |
| `-m, --model NAME` | `base` | `tiny` / `base` / `small` / `medium` / `large-v3-turbo` |
| `-l, --language CODE` | `auto` | ISO 639-1 code (`zh`, `en`, `ja`, …) or `auto` |
| `-o, --output DIR` | alongside input | Created if missing |
| `--format FMT[,FMT]` | `txt` | `txt` / `srt` / `json` / `md` / `all`, comma separated |
| `--config FILE` | `~/.config/transcriber/config.yaml` | Missing file means built-in defaults |
| `--threads N` | `0` | `0` = auto-detect |
| `--gpu BACKEND` | `auto` | `auto` / `metal` / `cuda` / `vulkan` / `cpu` |
| `--skip-existing` | off on the CLI | Config default is already `true`, so this flag only re-asserts it |
| `--initial-prompt TEXT` | empty | Decoder context: names, jargon, punctuation style |
| `--temperature FLOAT` | `0.0` | `0.0`–`1.0` |
| `--suppress-non-speech` | off | Suppress non-speech tokens |
| `--no-speech-threshold FLOAT` | `0.6` | `0.0`–`1.0` |
| `--max-segment-length N` | `0` | Characters per segment; `0` = no limit |
| `--split-on-word` | off | Split at word boundaries instead of characters |
| `--meta KEY=VALUE` | – | Repeatable; goes to Markdown front matter / JSON |
| `--no-streaming` | streaming on | Buffer and write the files after transcription instead |
| `init [--path FILE]` | `~/.config/transcriber/config.yaml` | Write the default config; refuses to overwrite an existing file |

`transcriber --help` lists the same set, and `transcriber --version` prints the build
version derived from git.

## Configuration file

```bash
transcriber init                 # write ~/.config/transcriber/config.yaml
transcriber init --path my.yaml  # or somewhere else
transcriber -i video.mp4 --config my.yaml
```

Precedence is **CLI flag > config file > built-in default**. A missing config file is
not an error — defaults are used.

```yaml
model:
  name: base                  # tiny/base/small/medium/large-v3-turbo
  language: auto              # ISO 639-1 or auto
  quantization:               # q4_k/q5_k/q6_k/q8_0; empty = F16
audio:
  sample_rate: 16000          # fixed, Whisper's input rate
  channels: 1                 # fixed, mono
performance:
  threads: 0                  # 0 = auto
  gpu: auto                   # auto/metal/cuda/vulkan/cpu
output:
  formats: [txt]              # txt/srt/json/md/all
  directory: "./"             # "./" = next to the input file
  skip_existing: true
  streaming: true             # write segments as they are transcribed
logging:
  level: info                 # trace/debug/info/warn
  file: ""                    # path for file logging; empty = stderr only
  colors: true
cache:
  directory: ~/.cache/whisper
inference:
  initial_prompt: ""
  temperature: 0.0
  suppress_non_speech: false
  no_speech_threshold: 0.6
  max_segment_length: 0
  split_on_word: false
custom_metadata:              # Markdown front matter, same as --meta
  speaker: "Ada"
```

These settings have **no CLI equivalent** and can only be changed in the config file:
`logging.*`, `cache.directory`, `model.quantization`, `custom_metadata`,
`audio.*`, and `output.streaming` (its CLI flag, `--no-streaming`, only turns it off).

`transcriber init` writes this file with the built-in defaults — plain YAML, no comments.
It refuses to overwrite an existing file, so delete it first or pass `--path`.

## Models

Models are downloaded on first use from
[HuggingFace `ggerganov/whisper.cpp`](https://huggingface.co/ggerganov/whisper.cpp)
into `~/.cache/whisper` and reused afterwards.

| Model | Download (F16) | Notes |
|-------|----------------|-------|
| `tiny` | 75 MB | Fastest, lowest accuracy |
| `base` | 148 MB | Default; good for clean speech |
| `small` | 488 MB | Noticeably better on accents |
| `medium` | 1.5 GB | High quality, slower |
| `large-v3-turbo` | 800 MB | Best quality/speed tradeoff |

Quantized variants (`ggml-<model>-q4_k.bin` and so on) are smaller and faster at some
accuracy cost. Set them in the config, not on the command line:

```yaml
model:
  name: large-v3-turbo
  quantization: q5_k     # q4_k / q5_k / q6_k / q8_0
```

To run fully offline, pre-place the same `.bin` files in the cache directory — the
tool only downloads when the file is missing.

## Output formats

| Format | Content |
|--------|---------|
| `txt` | One segment per line, no timestamps |
| `srt` | SubRip subtitles with `HH:MM:SS,mmm` timestamps |
| `json` | Full transcript object: segments, word-level timing, media metadata, custom metadata |
| `md` | YAML front matter + plain text body |

`md` front matter carries the auto-detected media facts (`ffprobe`), the transcription
summary, and your `--meta` values:

```markdown
---
source:
  filename: talk.mp4
  size_bytes: 94812304
  format: mp4
  bitrate: 415000
  duration_secs: 1834.5
video:
  codec: h264
  width: 1920
  height: 1080
  fps: 30.0
audio:
  codec: aac
  sample_rate: 48000
  channels: 2
transcription:
  model: medium
  language: zh
  duration: 1834.5
  transcribed_at: "2026-05-01T09:12:44Z"
custom_metadata:
  speaker: Ada
---

第一段文本

第二段文本
```

`json` is the same information plus `segments[]`, each with `start`, `end`, `text`, and
`words[]` (word, start, end).

Streaming is on by default: segments are flushed to disk as they are transcribed, so a
long file is not held in memory and an interrupted run leaves partial output.
`--no-streaming` writes everything at the end instead.

## Troubleshooting

| Symptom | Cause / fix |
|---------|-------------|
| `ffmpeg not found` | Install FFmpeg or add it to `PATH` (the error message prints per-platform install commands) |
| A folder run reports nothing | Files already have `.transcript.*` output and `skip_existing` is on; delete them or set `output.skip_existing: false` |
| `Unknown output format` | `--format` accepts only `txt`/`srt`/`json`/`md`/`all` |
| No GPU speedup | Check the backend was compiled in and selected; `--gpu cpu` forces the CPU path, which is the reliable fallback |
| Model download hangs or fails | Behind a proxy or offline: pre-place `ggml-<model>.bin` (or `ggml-<model>-<quant>.bin`) in `~/.cache/whisper` |
| Output in the wrong place | Default is next to the input file; use `-o DIR` to move it |

## Development

```bash
cargo test            # unit + integration tests in tests/
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
```

A pre-commit hook in `.githooks/` runs `cargo fmt` (and `taplo` for TOML when
installed); enable it with `git config core.hooksPath .githooks`.
Design notes and archived specs live in `openspec/`.

See [RELEASE.md](RELEASE.md) for the tagging and release process.

## License

MIT — see [LICENSE](LICENSE).

---

中文文档：[README_zh.md](README_zh.md)
