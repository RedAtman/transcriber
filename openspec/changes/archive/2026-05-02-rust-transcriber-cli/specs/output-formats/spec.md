## ADDED Requirements

### Requirement: TXT output format

The system SHALL output plain text transcript with one segment per line.

#### Scenario: Simple text output
- **WHEN** user requests txt output format
- **THEN** system creates `{stem}.transcript.txt` with segments separated by newlines

#### Scenario: TXT file content
```txt
这是第一段识别的文本。
这是第二段识别的文本。
第三段文本内容。
```

### Requirement: SRT subtitle format

The system SHALL output SRT (SubRip) subtitle format with timestamps.

#### Scenario: SRT output with timestamps
- **WHEN** user requests srt output format
- **THEN** system creates `{stem}.transcript.srt` with proper SRT formatting

#### Scenario: SRT file content example
```srt
1
00:00:00,000 --> 00:00:03,500
这是第一段识别的文本。

2
00:00:03,500 --> 00:00:07,200
这是第二段识别的文本。
```

### Requirement: JSON structured output

The system SHALL output JSON format with full metadata and word-level timing.

#### Scenario: JSON output with segments
- **WHEN** user requests json output format
- **THEN** system creates `{stem}.transcript.json` with structured data

#### Scenario: JSON file content structure
```json
{
  "file": "video.mp4",
  "model": "base",
  "language": "zh",
  "duration": 125.5,
  "segments": [
    {
      "start": 0.0,
      "end": 3.5,
      "text": "这是第一段识别的文本。",
      "words": [
        {"word": "这是", "start": 0.0, "end": 0.5},
        {"word": "第一段", "start": 0.5, "end": 1.2}
      ]
    }
  ],
  "transcribed_at": "2026-05-01T10:30:00Z"
}
```

### Requirement: Multiple output format selection

The system SHALL allow selecting multiple output formats simultaneously.

#### Scenario: Output all formats
- **WHEN** user sets `formats: ["all"]`
- **THEN** system creates txt, srt, and json files

#### Scenario: Output specific formats
- **WHEN** user sets `formats: ["txt", "json"]`
- **THEN** system creates only txt and json files

### Requirement: Output directory control

The system SHALL support custom output directory for transcript files.

#### Scenario: Same directory as input
- **WHEN** user does not specify output directory
- **THEN** transcript is saved in the same directory as video file

#### Scenario: Custom output directory
- **WHEN** user sets `output.directory: ./transcripts`
- **THEN** all transcripts are saved to that directory

### Requirement: Filename pattern

The system SHALL support custom filename patterns for output files.

#### Scenario: Default filename pattern
- **WHEN** processing `video.mp4`
- **THEN** output is `video.transcript.txt`

#### Scenario: Custom filename variables
- **WHEN** user specifies filename pattern with variables
- **THEN** system substitutes: `{stem}`, `{date}`, `{time}`

### Requirement: Skip existing transcripts

The system SHALL optionally skip files that already have transcripts.

#### Scenario: Skip existing with txt format
- **WHEN** `skip_existing: true` and `{stem}.transcript.txt` exists
- **THEN** system skips that video without processing

#### Scenario: Skip existing with multiple formats
- **WHEN** `skip_existing: true` and any transcript exists
- **THEN** system skips that video