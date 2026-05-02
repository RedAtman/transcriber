## Why

Currently, the transcriber writes all output files **after** the entire audio file has been transcribed. For long recordings (30+ minutes), users see zero output until transcription completes — a poor UX with no visibility into progress or partial results. Streaming output writes each transcribed segment to the output file immediately as it becomes available, giving users progressive visibility and partial results even if the process is interrupted.

## What Changes

- **New `output.streaming` config field** (default: `true`) controlling streaming mode
- **New `--no-streaming` CLI flag** to disable streaming (batch all-at-once mode)
- **Segment-by-segment file writing** within the transcription loop: TXT/SRT/JSON all get per-segment appending
- **Partial JSON array format**: write `[\n` header, each segment as `{...},\n`, then `]\n` footer
- **Survivable partial output**: on Ctrl+C or crash, the output file contains all segments transcribed so far (JSON may have trailing comma, trivially fixable)
- No breaking changes to CLI or config interface — all existing flags and YAML fields remain unchanged

## Capabilities

### New Capabilities
- `streaming-output`: progressive segment-by-segment writing to output files during transcription, with format-appropriate streaming for TXT, SRT, and JSON

### Modified Capabilities
*(None — all existing capabilities remain behaviorally unchanged when streaming is disabled)*

## Impact

- **`src/config.rs`**: `OutputConfig` gets `streaming: bool` field (default: `true`)
- **`src/cli.rs`**: New `--no-streaming` flag
- **`src/transcription.rs`**: `transcribe_with_whisper()` opens output files and writes segments during the iteration loop
- **`src/output.rs`**: New streaming helper functions (open stream, append segment, finalize) per format
- **`src/main.rs`**: `handle_transcribe()` skips `write_output()` call when streaming is enabled (files already written)
