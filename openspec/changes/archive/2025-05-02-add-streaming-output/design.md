## Context

The transcriber currently processes audio files through a pipeline: extract audio (FFmpeg) → load model → run whisper inference → iterate segments → write all output files at once. The `write_output()` method receives a complete `Transcript` struct and writes all format files in batch. For long recordings (30+ minutes), users see zero output until the entire inference completes.

The codebase uses whisper-rs v0.16.0. Its `full()` API is synchronous and blocking — it processes the entire audio buffer before returning. However, after `full()` returns, the segment iteration loop (transcription.rs lines 159-203) iterates through each decoded segment. This is the natural injection point for streaming output.

**Constraints:**
- whisper-rs has no async/non-blocking inference API
- Output formats: TXT (line-based), SRT (block-based), JSON (structured)
- Current `FormatWriter` trait takes full `&Transcript`, produces `Result<String>`
- The segment loop already has a progress bar callback per-segment

## Goals / Non-Goals

**Goals:**
- Write each transcribed segment to output files immediately during the segment iteration loop
- TXT: each segment appended as `segment.text + "\n"`
- SRT: each segment appended as a complete subtitle block
- JSON: write as partial JSON array — `[\n` header, `{...},\n` per segment, `]\n` footer
- Default mode is streaming (config: `output.streaming: true`)
- `--no-streaming` flag restores batch all-at-once behavior
- Survivable partial output: on Ctrl+C or crash, all transcribed segments are already in the file
- All existing CLI flags and YAML fields remain unchanged

**Non-Goals:**
- Real-time microphone streaming (not in scope)
- Sliding window / chunked audio processing (not needed for file-based transcription)
- VAD integration (not needed for file-based transcription)
- NDJSON or JSON Lines format (decided against, use partial JSON array)
- Network/WebSocket streaming (file output only)

## Decisions

### Decision 1: Loop-based writing (not callback-based)
**Chosen**: Write segments during the post-`full()` iteration loop (transcription.rs lines 159-203).

**Why**: whisper-rs has `set_segment_callback_safe()` that fires during `full()`, but:
- Callbacks fire from C code with opaque data — harder to debug
- The existing segment loop already has the progress bar hook — co-locating file I/O keeps logic together
- No observable UX difference: both approaches write after segments are decoded, since `full()` must complete before iteration

### Decision 2: Streaming helpers alongside FormatWriter (not modifying the trait)
**Chosen**: Add free functions `open_stream_output()`, `append_segment_to_stream()`, `finalize_stream_output()` in `output.rs`.

**Why**: The `FormatWriter` trait's `write(&self, transcript: &Transcript) -> Result<String>` is designed for batch mode. Changing it to support streaming would break the trait contract. Separate streaming functions are simpler and keep batch mode fully intact.

### Decision 3: JSON as partial array
**Chosen**: Write `[\n` → for each segment: `  {json},\n` → `]\n`.

**Why**: Matches the current JSON output structure. On crash, a trailing comma is present but trivially fixable (remove last comma, add `]`). The file is recoverable. This is the standard approach for streaming JSON arrays in CLI tools.

### Decision 4: `--no-streaming` flag (not `--streaming`)
**Chosen**: Opt-out flag rather than opt-in.

**Why**: Per user requirement, streaming is the default. An opt-out flag (`--no-streaming`) is cleaner than making users add `--streaming` every time. The config field is `output.streaming: bool` (default: true) for consistency.

### Decision 5: Streaming logic lives in `transcribe_with_whisper()`, not `transcribe_file()`
**Chosen**: Pass output config into `transcribe_with_whisper()` for file I/O during the segment loop.

**Why**: The segment loop is inside `transcribe_with_whisper()`. Writing there means we don't need to refactor the function boundary. `transcribe_file()` remains the public API; `transcribe_with_whisper()` handles file I/O as a side effect when streaming is enabled.

## Risks / Trade-offs

| Risk | Mitigation |
|------|------------|
| File I/O slows down segment iteration | File writes are buffered (std::fs::File uses BufWriter). For the segment counts involved (typically hundreds to low thousands), I/O is negligible vs inference time. |
| JSON partial array with trailing comma on crash | The file is still recoverable (remove last `,\n`, add `]\n`). A recovery note will be documented. |
| Streaming writes conflict with `write_output()` call | When streaming is enabled, `handle_transcribe()` skips the `write_output()` call entirely. No double-write risk. |
| Mixing streaming and non-streaming per format | Not supported — streaming applies to all formats uniformly. If streaming is enabled, all configured formats stream. |
