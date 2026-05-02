## ADDED Requirements

### Requirement: Video file transcription

The system SHALL accept a video file path and transcribe its audio content to text with word-level timing information.

#### Scenario: Single file transcription with default settings
- **WHEN** user runs `transcriber -i video.mp4` with no other arguments
- **THEN** system loads default model, extracts audio, transcribes, and outputs transcript.txt in the same directory

#### Scenario: Single file transcription with specified model and language
- **WHEN** user runs `transcriber -i video.mp4 -m medium -l zh`
- **THEN** system uses medium model for Chinese transcription

#### Scenario: Transcription with quantization
- **WHEN** user configures `quantization: q5_k` and runs transcription
- **THEN** system loads quantized model for faster inference with minimal quality loss

### Requirement: Directory batch transcription

The system SHALL process all video files in a specified directory recursively.

#### Scenario: Batch transcription with default settings
- **WHEN** user runs `transcriber -d ./videos`
- **THEN** system finds all .mp4 files and transcribes each one

#### Scenario: Batch transcription with skip-existing
- **WHEN** user runs `transcriber -d ./videos --skip-existing`
- **THEN** system skips any video that already has a corresponding transcript file

#### Scenario: Batch transcription with custom output directory
- **WHEN** user runs `transcriber -d ./videos -o ./output`
- **THEN** system outputs all transcripts to the specified directory

### Requirement: Progress reporting

The system SHALL display real-time progress during transcription.

#### Scenario: Single file progress
- **WHEN** transcribing a single file
- **THEN** system displays percentage complete, current phase (encoding/decoding), and ETA

#### Scenario: Batch progress with file counter
- **WHEN** transcribing multiple files
- **THEN** system displays `[N/M]` file counter with overall progress

### Requirement: GPU acceleration

The system SHALL automatically detect and utilize available GPU accelerators.

#### Scenario: Metal acceleration on Apple Silicon
- **WHEN** running on macOS with M1/M2/M3/M4 chip
- **THEN** system uses Metal backend for 3-4x speedup

#### Scenario: Vulkan acceleration on NVIDIA
- **WHEN** running on Linux with NVIDIA GPU
- **THEN** system uses Vulkan backend (faster than CUDA)

#### Scenario: CPU fallback
- **WHEN** no GPU is available
- **THEN** system uses multi-threaded CPU with SIMD acceleration

### Requirement: Thread count configuration

The system SHALL respect user-specified thread count for CPU processing.

#### Scenario: Explicit thread count
- **WHEN** user sets `threads: 8` in config
- **THEN** system uses exactly 8 threads

#### Scenario: Auto thread detection
- **WHEN** user sets `threads: 0` (default)
- **THEN** system auto-detects optimal thread count based on CPU cores