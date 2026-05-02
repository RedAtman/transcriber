## ADDED Requirements

### Requirement: FFmpeg audio extraction

The system SHALL extract audio from video files using FFmpeg.

#### Scenario: Extract audio with default settings
- **WHEN** transcribing a video file
- **THEN** system extracts audio to WAV format (16kHz, mono, PCM)

#### Scenario: FFmpeg audio parameters
- **WHEN** extracting audio
- **THEN** system uses: codec=pcm_s16le, sample_rate=16000, channels=1

### Requirement: FFmpeg detection and fallback

The system SHALL automatically detect and use available FFmpeg installation.

#### Scenario: Use system FFmpeg
- **WHEN** system FFmpeg is available in PATH
- **THEN** system uses it for audio extraction

#### Scenario: Fallback when FFmpeg unavailable
- **WHEN** system FFmpeg is not found
- **THEN** system reports clear error: "FFmpeg not found. Please install FFmpeg or ensure it is in PATH."

#### Scenario: Bundled FFmpeg (future consideration)
- **WHEN** bundled FFmpeg is available (future feature)
- **THEN** system uses bundled version as fallback

### Requirement: Temporary file management

The system SHALL manage temporary audio files during transcription.

#### Scenario: Create temporary directory
- **WHEN** transcribing a file
- **THEN** system creates a temporary directory for intermediate files

#### Scenario: Cleanup on success
- **WHEN** transcription completes successfully
- **THEN** system removes temporary audio files

#### Scenario: Cleanup on failure
- **WHEN** transcription fails
- **THEN** system still removes temporary files in finally block

### Requirement: Audio format compatibility

The system SHALL support common video formats as input.

#### Scenario: MP4 input
- **WHEN** input is .mp4 file
- **THEN** system extracts audio successfully

#### Scenario: Other video formats
- **WHEN** input is .mov, .avi, .mkv, or .webm
- **THEN** system extracts audio if FFmpeg supports the format

#### Scenario: Unsupported format
- **WHEN** input is an unsupported format
- **THEN** system reports error with format information