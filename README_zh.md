# transcriber

基于 [whisper.cpp](https://github.com/ggerganov/whisper.cpp) 的快速视频转录 CLI。
给一个视频文件或一个文件夹，它用 FFmpeg 抽出音频，在本机用 GPU（Metal / CUDA / Vulkan）
或 CPU 转录，输出 TXT、SRT、JSON 或 Markdown。

音频不会离开本机，唯一的网络访问是首次下载模型。

## 快速开始

```bash
brew install ffmpeg                              # 运行时必需
brew tap RedAtman/tap && brew install transcriber

transcriber -i video.mp4                         # → video.transcript.txt
transcriber -d ./recordings -l zh --format srt   # 整个文件夹，中文，字幕
```

## 安装

### 1. FFmpeg（必需）

`ffmpeg` 和 `ffprobe` 必须在 `PATH` 中，缺失时工具会直接拒绝启动。

| 平台 | 命令 |
|------|------|
| macOS | `brew install ffmpeg` |
| Debian / Ubuntu | `sudo apt install ffmpeg` |
| Windows | 从 <https://ffmpeg.org/download.html> 下载并加入 `PATH` |

### 2. transcriber

**Homebrew（macOS、Linux）：**

```bash
brew tap RedAtman/tap
brew install transcriber
```

**预编译二进制** —— 从 [Releases](https://github.com/RedAtman/transcriber/releases/latest)
下载对应平台的资产，放到 `PATH` 里：

| 平台 | 资产名 |
|------|--------|
| macOS Apple Silicon | `transcriber-macos-aarch64` |
| macOS Intel | `transcriber-macos-x86_64` |
| Linux x86_64 | `transcriber-linux-x86_64` |
| Windows x86_64 | `transcriber-windows-x86_64.exe` |

**从源码构建：**

```bash
git clone https://github.com/RedAtman/transcriber.git
cd transcriber
cargo build --release       # 二进制位于 ./target/release/transcriber
```

构建过程会编译 whisper.cpp，因此需要 C/C++ 工具链以及下列系统库——与 release
构建安装的是同一批：

| 平台 | 软件包 |
|------|--------|
| macOS | `brew install cmake pkg-config opus libsndfile libsamplerate` |
| Debian / Ubuntu | `sudo apt install build-essential cmake pkg-config libopus-dev libasound2-dev libsamplerate0-dev libssl-dev` |

## 使用方法

### 转录单个文件

```bash
transcriber -i video.mp4                       # 默认模型（base）、自动识别语言、txt
transcriber -i video.mp4 -m medium -l zh       # 更高质量模型，中文
transcriber -i video.mp4 -o ./subtitles --format srt
```

### 转录文件夹

`-d` 会递归子目录，识别 `mp4`、`mov`、`avi`、`mkv`、`webm`、`m4v`（不区分大小写）。
`-d` 与 `-i` 互斥。

```bash
transcriber -d ./videos
transcriber -d ./videos --format txt,srt,json
transcriber -d ./videos --format all           # txt + srt + json + md
```

已有输出的文件默认会被跳过（配置中 `output.skip_existing: true`）。要重新转录，
可以删掉已有的 `.transcript.*` 文件，或在配置里设 `output.skip_existing: false`
——命令行没有关闭它的开关。

### 输出去哪里

每种格式写到 `<文件名主干>.transcript.<扩展名>`，主干是去掉扩展名的输入文件名：

```
recordings/2026-05-01-talk.mkv
  → recordings/2026-05-01-talk.transcript.txt
  → recordings/2026-05-01-talk.transcript.srt
```

默认与输入文件同目录。`--output DIR`（或配置里的 `output.directory`）会改写到
`DIR`，目录不存在时自动创建。

### 自定义元信息

`--meta` 可重复使用，会写进 Markdown 的 YAML 头部（同时出现在 JSON 输出里）。
含空格的值需要加引号。

```bash
transcriber -i talk.mp4 --format md \
  --meta title="季度复盘" --meta speaker="Ada" --meta location=北京
```

### 解码参数调优

```bash
transcriber -i talk.mp4 --initial-prompt "Kubernetes, gRPC, 可观测性"
transcriber -i talk.mp4 --temperature 0.2          # 0.0 = 确定性输出（默认）
transcriber -i talk.mp4 --suppress-non-speech      # 抑制咳嗽、背景噪声等标记
transcriber -i talk.mp4 --no-speech-threshold 0.4  # 更激进地判定静音
transcriber -i talk.mp4 --max-segment-length 42 --split-on-word
```

## 命令行参数

| 参数 | 默认值 | 说明 |
|------|--------|------|
| `-i, --input FILE` | – | 单文件输入，与 `-d` 互斥 |
| `-d, --dir DIR` | – | 递归文件夹，与 `-i` 互斥 |
| `-m, --model NAME` | `base` | `tiny` / `base` / `small` / `medium` / `large-v3-turbo` |
| `-l, --language CODE` | `auto` | ISO 639-1（`zh`、`en`、`ja`…）或 `auto` |
| `-o, --output DIR` | 输入文件同目录 | 不存在时自动创建 |
| `--format FMT[,FMT]` | `txt` | `txt` / `srt` / `json` / `md` / `all`，逗号分隔 |
| `--config FILE` | `~/.config/transcriber/config.yaml` | 文件不存在则用内置默认值 |
| `--threads N` | `0` | `0` = 自动探测 |
| `--gpu BACKEND` | `auto` | `auto` / `metal` / `cuda` / `vulkan` / `cpu` |
| `--skip-existing` | 命令行默认关闭 | 配置里默认已是 `true`，该参数只是再次确认 |
| `--initial-prompt TEXT` | 空 | 解码上下文：人名、术语、标点风格 |
| `--temperature FLOAT` | `0.0` | `0.0`–`1.0` |
| `--suppress-non-speech` | 关闭 | 抑制非语音标记 |
| `--no-speech-threshold FLOAT` | `0.6` | `0.0`–`1.0` |
| `--max-segment-length N` | `0` | 每段字符数上限，`0` = 不限制 |
| `--split-on-word` | 关闭 | 按单词边界而非字符切分 |
| `--meta KEY=VALUE` | – | 可重复，写入 Markdown 头部 / JSON |
| `--no-streaming` | 默认开启流式 | 改为转录结束后一次性写出文件 |
| `init [--path FILE]` | `~/.config/transcriber/config.yaml` | 生成默认配置；已有文件时不会覆盖 |

`transcriber --help` 列出同一套参数，`transcriber --version` 打印由 git 派生的版本号。

## 配置文件

```bash
transcriber init                 # 写入 ~/.config/transcriber/config.yaml
transcriber init --path my.yaml  # 或写到别处
transcriber -i video.mp4 --config my.yaml
```

优先级为 **命令行参数 > 配置文件 > 内置默认值**。配置文件不存在不算错误，直接使用默认值。

```yaml
model:
  name: base                  # tiny/base/small/medium/large-v3-turbo
  language: auto              # ISO 639-1 或 auto
  quantization:               # q4_k/q5_k/q6_k/q8_0，留空 = F16
audio:
  sample_rate: 16000          # 固定值，Whisper 的输入采样率
  channels: 1                 # 固定值，单声道
performance:
  threads: 0                  # 0 = 自动
  gpu: auto                   # auto/metal/cuda/vulkan/cpu
output:
  formats: [txt]              # txt/srt/json/md/all
  directory: "./"             # "./" = 与输入文件同目录
  skip_existing: true
  streaming: true             # 逐段写盘
logging:
  level: info                 # trace/debug/info/warn
  file: ""                    # 日志文件路径，留空则只输出到 stderr
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
custom_metadata:              # 写入 Markdown 头部，等价于 --meta
  speaker: "Ada"
```

以下设置**没有命令行等价参数**，只能改配置文件：`logging.*`、`cache.directory`、
`model.quantization`、`custom_metadata`、`audio.*`，以及 `output.streaming`
（它的命令行开关 `--no-streaming` 只能关闭）。

`transcriber init` 写出的就是这份文件的内置默认值——纯 YAML，没有注释。
它不会覆盖已存在的文件，需要先删除或用 `--path` 指定别处。

## 模型

模型在首次使用时从
[HuggingFace `ggerganov/whisper.cpp`](https://huggingface.co/ggerganov/whisper.cpp)
下载到 `~/.cache/whisper`，之后复用。

| 模型 | 下载大小（F16） | 说明 |
|------|----------------|------|
| `tiny` | 75 MB | 最快，准确率最低 |
| `base` | 148 MB | 默认，干净语音够用 |
| `small` | 488 MB | 口音场景明显更好 |
| `medium` | 1.5 GB | 高质量，较慢 |
| `large-v3-turbo` | 800 MB | 质量/速度折中最好 |

量化版本（`ggml-<model>-q4_k.bin` 等）体积更小、速度更快，准确率略有损失。
只能在配置里指定，命令行没有对应参数：

```yaml
model:
  name: large-v3-turbo
  quantization: q5_k     # q4_k / q5_k / q6_k / q8_0
```

要完全离线运行，把同样的 `.bin` 文件预先放进缓存目录即可——只有文件缺失时才会联网下载。

## 输出格式

| 格式 | 内容 |
|------|------|
| `txt` | 每段一行，无时间戳 |
| `srt` | SubRip 字幕，`HH:MM:SS,mmm` 时间戳 |
| `json` | 完整转录对象：分段、词级时间、媒体元信息、自定义元信息 |
| `md` | YAML 头部 + 纯文本正文 |

`md` 的头部包含 ffprobe 自动探测的媒体信息、转录摘要，以及 `--meta` 的内容：

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

`json` 包含同样的信息，外加 `segments[]`，每段有 `start`、`end`、`text` 和
`words[]`（word、start、end）。

流式输出默认开启：每转录出一段就写盘，长文件不会整体驻留内存，中断时也会留下部分结果。
`--no-streaming` 改为结束时一次性写入。

## 故障排查

| 现象 | 原因 / 处理 |
|------|-------------|
| `ffmpeg not found` | 安装 FFmpeg 或把它加入 `PATH`（报错信息会给出各平台安装命令） |
| 文件夹模式什么都没做 | 已有 `.transcript.*` 输出且 `skip_existing` 开启；删掉产物或设 `output.skip_existing: false` |
| `Unknown output format` | `--format` 只接受 `txt`/`srt`/`json`/`md`/`all` |
| 没有 GPU 加速 | 确认该后端已编译进二进制并被选中；`--gpu cpu` 是可靠的 CPU 回退 |
| 模型下载卡住或失败 | 代理或离线环境：把 `ggml-<model>.bin`（或 `ggml-<model>-<quant>.bin`）预放进 `~/.cache/whisper` |
| 输出位置不对 | 默认与输入文件同目录，用 `-o DIR` 指定 |

## 开发

```bash
cargo test            # 单元测试 + tests/ 下的集成测试
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
```

`.githooks/` 里的 pre-commit 钩子会跑 `cargo fmt`（装了 `taplo` 时也格式化 TOML），
用 `git config core.hooksPath .githooks` 启用。设计说明与已归档的规格在 `openspec/`。

打标签与发版流程见 [RELEASE.md](RELEASE.md)。

## 许可证

MIT —— 见 [LICENSE](LICENSE)。

---

English documentation: [README.md](README.md)
