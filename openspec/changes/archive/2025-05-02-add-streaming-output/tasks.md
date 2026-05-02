## 1. Config & CLI

- [ ] 1.1 Add `streaming: bool` field to `OutputConfig` (default: `true`) in `config.rs`, update `Default` impl
- [ ] 1.2 Add `--no-streaming` CLI flag to `Cli` struct in `cli.rs`
- [ ] 1.3 Add `streaming: Option<bool>` to `CliOverrides`, update `From<&Cli>` and `merge_with_cli()`

## 2. Output Module: Streaming Primitives

- [ ] 2.1 Create `StreamOutput` struct with `file: BufWriter<File>`, `format: String`, `segment_count: usize`
- [ ] 2.2 Implement `open_stream_outputs(stem, formats, output_dir)` — opens files, writes JSON `[\n` header
- [ ] 2.3 Implement `append_segment_to_streams(streams, segment)` — writes segment to each stream in format-appropriate way (TXT line, SRT block, JSON array element with commas)
- [ ] 2.4 Implement `finalize_stream_outputs(streams)` — writes JSON `\n]` footer, flushes files

## 3. Transcription: Wire Streaming Into Pipeline

- [ ] 3.1 Update `transcribe_with_whisper()` to accept `Option<&mut Vec<StreamOutput>>`, call `append_segment_to_streams()` in the segment loop
- [ ] 3.2 Update `Transcriber::transcribe_file()` to open streams before transcription, finalize after
- [ ] 3.3 Handle `"all"` format expansion for streaming (same as `write_output()` does)
- [ ] 3.4 In `handle_transcribe()` (main.rs) and `Transcriber::transcribe_directory()`, skip `write_output()` when streaming is enabled

## 4. Verification

- [ ] 4.1 `cargo build --release` compiles, `cargo test` passes
- [ ] 4.2 Default config includes `streaming: true`, `transcriber init` generates correct config
- [ ] 4.3 Streaming TXT/SRT/JSON output matches expected format (segment-by-segment appending)
- [ ] 4.4 `--no-streaming` produces byte-identical output to pre-streaming version
- [ ] 4.5 Verify partial JSON output is recoverable (trailing comma + missing `]`)
