## ADDED Requirements

### Requirement: Streaming is default mode
The system SHALL enable streaming output by default. The `output.streaming` config field SHALL default to `true`.

#### Scenario: Default config has streaming enabled
- **WHEN** a user runs `transcriber init` to generate a default config
- **THEN** the generated config SHALL contain `streaming: true` under the `output` section

#### Scenario: --no-streaming disables streaming
- **WHEN** a user passes `--no-streaming` on the CLI
- **THEN** the system SHALL write all output files in batch mode after transcription completes, as before

### Requirement: TXT streaming format
When streaming is enabled with TXT format, the system SHALL append each segment's text as a separate line to the `.txt` file immediately after the segment is transcribed.

#### Scenario: TXT file grows per segment
- **WHEN** transcription produces segment "Hello world" (segment 0) and then "Goodbye" (segment 1)
- **THEN** after segment 0, the .txt file SHALL contain "Hello world\n"
- **THEN** after segment 1, the .txt file SHALL contain "Hello world\nGoodbye\n"

### Requirement: SRT streaming format
When streaming is enabled with SRT format, the system SHALL append each segment as a complete subtitle block to the `.srt` file immediately after the segment is transcribed.

#### Scenario: SRT file grows per segment
- **WHEN** transcription produces segment 0 (0.0-3.5s, "Hello") and then segment 1 (3.5-7.2s, "World")
- **THEN** after segment 0, the .srt file SHALL contain "1\n00:00:00,000 --> 00:00:03,500\nHello\n\n"
- **THEN** after segment 1, the .srt file SHALL contain the full content including both subtitle blocks

### Requirement: JSON streaming format
When streaming is enabled with JSON format, the system SHALL write the JSON output as a partial JSON array: `[\n` header, each segment as a JSON object line (with trailing comma), then `]\n` footer after the final segment.

#### Scenario: JSON file structure with streaming
- **WHEN** transcription produces 2 segments
- **THEN** the final .json file SHALL contain valid JSON with the structure `[{...},{...}]`
- **THEN** the .json file SHALL NOT use NDJSON or JSON Lines format

#### Scenario: JSON file is survivable on interruption
- **WHEN** transcription is interrupted by Ctrl+C after writing segment 0 but before segment 1
- **THEN** the .json file SHALL contain a recoverable partial JSON array (may have trailing comma after segment 0)

### Requirement: Streaming output file path
When streaming is enabled, the system SHALL use the same `output_file_path()` function to determine output paths, producing files at `{stem}.transcript.{format}`.

#### Scenario: Output file path matches batch mode
- **WHEN** streaming is enabled and a file named "lecture.mp4" is transcribed
- **THEN** the streaming TXT output SHALL be written to `{output_dir}/lecture.transcript.txt`
- **THEN** the streaming SRT output SHALL be written to `{output_dir}/lecture.transcript.srt`
- **THEN** the streaming JSON output SHALL be written to `{output_dir}/lecture.transcript.json`

### Requirement: No double-write in streaming mode
When streaming is enabled, the system SHALL NOT call the batch `write_output()` method, to prevent duplicate output.

#### Scenario: Batch write_output is skipped in streaming mode
- **WHEN** streaming is enabled and a file is transcribed
- **THEN** `write_output()` SHALL NOT be called after `transcribe_file()` returns
- **THEN** the output files SHALL contain complete transcript data (all segments written during iteration)

### Requirement: Batch mode is unchanged
When streaming is disabled (`--no-streaming` or `output.streaming: false`), the system SHALL behave exactly as before: `transcribe_file()` returns a full `Transcript`, and `write_output()` writes all files at once.

#### Scenario: --no-streaming produces identical output to previous version
- **WHEN** `--no-streaming` is passed and a file is transcribed
- **THEN** the output files SHALL be byte-identical to those produced by the transcriber before streaming was added (given the same inputs and config)
