## ADDED Requirements

### Requirement: Automatic model download

The system SHALL automatically download Whisper models from HuggingFace when not found locally.

#### Scenario: Download on first use
- **WHEN** user requests model `base` that is not cached
- **THEN** system downloads from HuggingFace and saves to cache directory

#### Scenario: Skip download if cached
- **WHEN** user requests model `base` that exists in cache
- **THEN** system loads from cache without downloading

#### Scenario: Download with progress
- **WHEN** downloading a model
- **THEN** system displays download progress with percentage

### Requirement: Model cache management

The system SHALL manage a local cache of downloaded models.

#### Scenario: Default cache location
- **WHEN** user does not specify cache directory
- **THEN** system uses `~/.cache/transcriber/`

#### Scenario: Custom cache directory
- **WHEN** user sets `cache.directory: /custom/path`
- **THEN** system downloads models to the specified directory

#### Scenario: Model verification
- **WHEN** loading a cached model
- **THEN** system verifies file integrity (unless skip_verify is true)

### Requirement: Supported model sizes

The system SHALL support all standard Whisper model sizes.

| Model | F16 Size | Description |
|-------|----------|-------------|
| tiny | 75 MB | Fastest, lowest quality |
| base | 148 MB | Default, balanced |
| small | 488 MB | Better quality |
| medium | 1.5 GB | High quality |
| large-v3-turbo | 800 MB | Best quality/speed ratio |

#### Scenario: Load tiny model
- **WHEN** user specifies `-m tiny`
- **THEN** system loads the tiny model

#### Scenario: Load large-v3-turbo model
- **WHEN** user specifies `-m large-v3-turbo`
- **THEN** system loads the large-v3-turbo model

### Requirement: Model quantization

The system SHALL support quantized models for reduced memory and faster inference.

#### Scenario: Use Q5_K quantization
- **WHEN** user sets `quantization: q5_k`
- **THEN** system uses or downloads the Q5_K quantized model variant

#### Scenario: Use F16 full precision
- **WHEN** user leaves quantization unset or sets `none`
- **THEN** system uses F16 model

#### Scenario: Quantization format options
- **WHEN** user specifies quantization
- **THEN** valid options are: q4_k, q5_k, q6_k, q8_0

### Requirement: Model fallback on GPU unavailable

The system SHALL fall back to CPU if the selected GPU backend is unavailable.

#### Scenario: Metal unavailable on Intel Mac
- **WHEN** Metal backend fails on non-Apple Silicon Mac
- **THEN** system falls back to CPU with Accelerate framework

#### Scenario: CUDA unavailable
- **WHEN** CUDA backend fails (no NVIDIA GPU)
- **THEN** system falls back to CPU with OpenMP