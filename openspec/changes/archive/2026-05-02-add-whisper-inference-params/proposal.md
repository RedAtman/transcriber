## Why

Whisper.cpp 支持丰富的推理参数（temperature, initial_prompt, suppress_non_speech 等），可以显著提升转录质量。当前 transcriber CLI 只暴露了基础的 model/language/threads 参数，用户无法配置这些高级选项来优化识别准确率或处理特殊场景（如噪声环境、专业术语识别）。

## What Changes

- 新增 `InferenceConfig` 配置结构，包含 6 个推理参数
- 配置文件中新增 `inference:` section，所有参数带默认值
- CLI 增加对应的命令行参数覆盖
- `transcribe_with_whisper()` 函数应用这些参数到 `FullParams`

## Capabilities

### New Capabilities

- `inference-params`: 转录推理参数配置，支持 initial_prompt、temperature、suppress_non_speech、no_speech_threshold、max_segment_length、split_on_word

### Modified Capabilities

- 无现有 spec 需要修改

## Impact

- `src/config.rs`: 新增 `InferenceConfig` struct，修改 `Config` 包含新 section
- `src/cli.rs`: 新增 CLI 参数选项
- `src/transcription.rs`: `transcribe_with_whisper()` 使用新参数
- 默认配置文件 `config.yaml` 包含新参数及默认值
- 用户可配置 inference 参数改善识别准确率