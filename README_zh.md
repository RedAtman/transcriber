# transcriber

基于 whisper.cpp 的快速视频转录 CLI 工具。从视频文件中提取音频，并利用 GPU 加速将其转录为文本。

## 功能特点

- **视频 → 音频 → 文本**：通过 FFmpeg 自动提取音频
- **GPU 加速**：支持 Metal (macOS M*)、Vulkan (NVIDIA)、CUDA 或 CPU 回退
- **多种输出格式**：TXT、SRT（字幕）、JSON（包含词级时间戳）
- **批量处理**：递归转录整个目录
- **模型量化**：Q4_K/Q5_K/Q6_K/Q8_0 精度级别，速度与大小可权衡
- **可配置**：YAML 配置文件，支持 CLI 参数覆盖
- **进度报告**：下载和转录过程实时显示进度条

## 安装

### 前置条件

- **FFmpeg**：音频提取必需
  - macOS：`brew install ffmpeg`
  - Ubuntu：`sudo apt install ffmpeg`
  - Windows：从 https://ffmpeg.org/ 下载

### 从源码构建

```bash
git clone <repo-url>
cd transcriber
cargo build --release
```

二进制文件位于 `./target/release/transcriber`。

## 使用方法

### 单文件转录

```bash
# 使用默认设置转录（base 模型，txt 输出）
transcriber -i video.mp4

# 指定模型和语言
transcriber -i video.mp4 -m medium -l zh

# 自定义输出目录和 SRT 格式
transcriber -i video.mp4 -o ./subtitles --format srt
```

### 批量处理

```bash
# 转录目录中的所有视频
transcriber -d ./videos

# 跳过已转录的文件
transcriber -d ./videos --skip-existing

# 多种输出格式
transcriber -d ./videos --format "txt,srt,json"
```

### 配置

```bash
# 生成默认配置文件
transcriber init

# 使用自定义配置
transcriber -i video.mp4 --config ./my-config.yaml
```

默认配置文件位置：`~/.config/transcriber/config.yaml`

### 可用模型

| 模型 | 大小 | 说明 |
|------|------|------|
| tiny | 75 MB | 最快，质量最低 |
| base | 148 MB | 默认，均衡 |
| small | 488 MB | 更好质量 |
| medium | 1.5 GB | 高质量 |
| large-v3-turbo | 800 MB | 最佳质量/速度比 |

## 输出格式

- **TXT**：纯文本，每段一行
- **SRT**：SubRip 字幕格式，包含时间戳
- **JSON**：结构化数据，包含元数据和词级时间

## 许可证

MIT