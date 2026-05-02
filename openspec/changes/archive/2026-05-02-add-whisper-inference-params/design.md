## Context

当前 transcriber 仅支持基础的 model/language/threads 配置。whisper.cpp 底层支持丰富的推理参数，用户无法配置这些来优化识别质量。

## Goals / Non-Goals

**Goals:**
- 支持 6 个常用推理参数：initial_prompt、temperature、suppress_non_speech、no_speech_threshold、max_segment_length、split_on_word
- 配置文件 + CLI 参数双重覆盖机制
- 所有参数有合理的默认值

**Non-Goals:**
- 不包含实验性功能（VAD、文法约束、tinydiarize）
- 不包含进阶调参（logprob_thold、entropy_thold 等）- 留待 v2

## Decisions

### 1. 独立 `inference:` 配置 section

```yaml
inference:
  initial_prompt: ""
  temperature: 0.0
  suppress_non_speech: false
  no_speech_threshold: 0.6
  max_segment_length: 0
  split_on_word: false
```

**原因**: 清晰分离模型配置和推理参数，避免 `model:` section 臃肿。用户可快速定位相关配置。

### 2. CLI 参数命名

| CLI 参数 | 对应字段 | 类型 |
|----------|----------|------|
| `--initial-prompt` | inference.initial_prompt | String |
| `--temperature` | inference.temperature | f32 |
| `--suppress-non-speech` | inference.suppress_non_speech | bool |
| `--no-speech-threshold` | inference.no_speech_threshold | f32 |
| `--max-segment-length` | inference.max_segment_length | u32 |
| `--split-on-word` | inference.split_on_word | bool |

### 3. 参数默认值

| 参数 | 默认值 | 原因 |
|------|--------|------|
| initial_prompt | "" (空) | 用户很少需要，必须显式配置 |
| temperature | 0.0 | whisper.cpp 默认，贪心采样，质量最高 |
| suppress_non_speech | false | 默认不抑制，用户按需开启 |
| no_speech_threshold | 0.6 | whisper.cpp 默认，合理的非语音过滤 |
| max_segment_length | 0 | 0=不限，由模型自己决定分段 |
| split_on_word | false | 默认按字符分割，split_on_word 需要额外计算 |

### 4. 配置合并逻辑

CLI 参数优先级最高，覆盖配置文件值。

## Risks / Trade-offs

- **参数组合不当时可能降低质量** → 用户需参考文档选择合适值（温度过高可能导致乱码）
- **新增参数增加配置复杂度** → 提供默认配置文件模板，所有参数带注释说明
- **initial_prompt 中文编码** → whisper-rs 使用 CString，确保 UTF-8 正确传递

## Migration Plan

1. 修改 `Config` 结构，添加 `inference: InferenceConfig`
2. 实现 `InferenceConfig` 及默认值
3. 添加 CLI 参数到 `CliOverrides`
4. 修改 `transcribe_with_whisper()` 应用参数
5. 更新 `transcriber init` 生成默认配置文件
6. 运行测试验证

## Open Questions

- 是否需要验证参数的合理范围？（如 temperature 必须在 0-1.0）→ 暂时不做，硬编码约束在代码里