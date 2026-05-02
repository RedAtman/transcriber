## ADDED Requirements

### Requirement: Inference parameters configuration

系统 SHALL 支持配置转录推理参数，通过配置文件 `inference:` section 或 CLI 参数覆盖。

#### Scenario: Default config generates inference section
- **WHEN** 用户执行 `transcriber init` 生成默认配置
- **THEN** 生成的配置文件包含完整的 `inference:` section，包含所有 6 个参数及其默认值

#### Scenario: YAML config loads inference parameters
- **WHEN** 配置文件中存在 `inference:` section 且参数格式正确
- **THEN** 系统正确解析并应用这些参数到转录过程

#### Scenario: CLI parameters override config values
- **WHEN** 用户同时指定了配置文件和 CLI 参数（如 `--temperature`）
- **THEN** CLI 参数值优先于配置文件值

### Requirement: initial_prompt parameter

系统 SHALL 支持 `initial_prompt` 参数，用于提供上下文提示词改善识别准确率。

#### Scenario: Empty initial_prompt by default
- **WHEN** 用户未配置 initial_prompt
- **THEN** 使用空字符串作为初始提示

#### Scenario: Initial prompt passed to whisper
- **WHEN** 用户设置 `--initial-prompt "以下是技术会议"`
- **THEN** whisper 模型使用此提示作为解码起始上下文

#### Scenario: Initial prompt with special characters
- **WHEN** 用户设置包含中文、标点的 initial_prompt
- **THEN** 系统正确处理 UTF-8 编码并传递给 whisper.cpp

### Requirement: temperature parameter

系统 SHALL 支持 `temperature` 参数控制采样随机性，默认值为 0.0。

#### Scenario: Zero temperature for deterministic output
- **WHEN** temperature 设置为 0.0
- **THEN** 使用贪心采样，输出确定性结果

#### Scenario: Temperature passed to whisper
- **WHEN** 用户设置 `--temperature 0.2`
- **THEN** whisper 使用指定的温度进行采样

### Requirement: suppress_non_speech parameter

系统 SHALL 支持 `suppress_non_speech` 参数，抑制非语音 token（如咳嗽声、背景音）。

#### Scenario: Disabled by default
- **WHEN** 用户未配置 suppress_non_speech
- **THEN** 参数值为 false，保留所有识别结果

#### Scenario: Enabled suppresses non-speech tokens
- **WHEN** 用户设置 `--suppress-non-speech`
- **THEN** whisper 抑制非语音 token 输出

### Requirement: no_speech_threshold parameter

系统 SHALL 支持 `no_speech_threshold` 参数，用于过滤低于置信度的语音段。

#### Scenario: Default threshold value
- **WHEN** 用户未配置 no_speech_threshold
- **THEN** 使用默认值 0.6

#### Scenario: Custom threshold affects silence detection
- **WHEN** 用户设置 `--no-speech-threshold 0.8`
- **THEN** whisper 使用 0.8 作为非语音检测阈值

### Requirement: max_segment_length parameter

系统 SHALL 支持 `max_segment_length` 参数限制单段最大字符数，默认 0 表示不限。

#### Scenario: No limit by default
- **WHEN** 用户未配置 max_segment_length
- **THEN** 参数值为 0，模型自行决定分段

#### Scenario: Maximum length enforced
- **WHEN** 用户设置 `--max-segment-length 100`
- **THEN** 每个转录 segment 不超过 100 个字符

### Requirement: split_on_word parameter

系统 SHALL 支持 `split_on_word` 参数，控制时间戳按词边界分割而非字符。

#### Scenario: Character-based split by default
- **WHEN** 用户未配置 split_on_word
- **THEN** 时间戳按字符边界分割

#### Scenario: Word-based split when enabled
- **WHEN** 用户设置 `--split-on-word`
- **THEN** 时间戳按词边界分割，提高可读性

### Requirement: Parameter validation

系统 SHALL 在启动时验证推理参数的合理范围。

#### Scenario: Valid temperature range
- **WHEN** temperature 在 0.0-1.0 范围内
- **THEN** 参数被接受并应用

#### Scenario: Invalid temperature rejected
- **WHEN** temperature 超出 0.0-1.0 范围
- **THEN** 系统输出警告并使用默认值 0.0

#### Scenario: Valid threshold range
- **WHEN** no_speech_threshold 在 0.0-1.0 范围内
- **THEN** 参数被接受并应用