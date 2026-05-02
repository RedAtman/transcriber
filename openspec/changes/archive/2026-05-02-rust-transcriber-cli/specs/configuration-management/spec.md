## ADDED Requirements

### Requirement: YAML configuration file loading

The system SHALL load configuration from YAML files at standard XDG locations.

#### Scenario: Load from user config directory
- **WHEN** user runs transcriber without --config flag
- **THEN** system loads `~/.config/transcriber/config.yaml` if it exists

#### Scenario: Load from explicit config path
- **WHEN** user runs `transcriber --config /path/to/config.yaml`
- **THEN** system loads from the specified path

#### Scenario: Fallback to defaults when no config exists
- **WHEN** no config file exists at any location
- **THEN** system uses built-in defaults

### Requirement: Configuration parameter structure

The configuration file SHALL support the following structure:

```yaml
model:
  name: base              # tiny/base/small/medium/large-v3-turbo
  language: zh            # ISO 639-1 code or "auto"
  quantization: q5_k       # optional: q4_k/q5_k/q6_k or none

audio:
  sample_rate: 16000      # fixed: 16kHz for Whisper
  channels: 1             # fixed: mono

performance:
  threads: 0              # 0 = auto detect
  gpu: auto               # auto/metal/cuda/vulkan/cpu

output:
  formats: ["txt"]        # txt/srt/json/all
  directory: "./"         # output directory
  skip_existing: true     # skip if transcript exists

logging:
  level: "info"           # trace/debug/info/warn
  file: ""                # optional log file path
  colors: true            # colored terminal output

cache:
  directory: "~/.cache/transcriber"  # model cache directory
```

#### Scenario: Valid configuration with all fields
- **WHEN** config.yaml contains all valid fields
- **THEN** system loads and applies all settings

#### Scenario: Partial configuration with defaults
- **WHEN** config.yaml contains only model.name
- **THEN** system uses default values for all other fields

### Requirement: CLI argument override

The system SHALL allow CLI arguments to override configuration file values.

#### Scenario: CLI overrides config file
- **WHEN** config.yaml has `model.name: base` and CLI has `-m medium`
- **THEN** system uses medium model

#### Scenario: Parameter priority order
- **WHEN** same parameter is set in config file and CLI
- **THEN** CLI value takes precedence

### Requirement: Configuration validation

The system SHALL validate configuration values and report errors for invalid settings.

#### Scenario: Invalid model name
- **WHEN** config has `model.name: invalid`
- **THEN** system exits with error: "Invalid model: invalid. Valid options: tiny, base, small, medium, large-v3-turbo"

#### Scenario: Invalid language code
- **WHEN** config has `model.language: invalid`
- **THEN** system exits with error explaining valid format

#### Scenario: Thread count exceeds CPU
- **WHEN** config has `threads: 999`
- **THEN** system warns but continues with adjusted count

### Requirement: Init command for configuration generation

The system SHALL provide a command to generate a default configuration file.

#### Scenario: Generate default config
- **WHEN** user runs `transcriber init`
- **THEN** system creates `~/.config/transcriber/config.yaml` with all default values

#### Scenario: Init with custom path
- **WHEN** user runs `transcriber init --path ./config.yaml`
- **THEN** system creates config at the specified path