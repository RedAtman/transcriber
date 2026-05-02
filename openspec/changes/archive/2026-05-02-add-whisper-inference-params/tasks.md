## 1. Config Struct

- [x] 1.1 Add `InferenceConfig` struct in `src/config.rs` with 6 fields: `initial_prompt`, `temperature`, `suppress_non_speech`, `no_speech_threshold`, `max_segment_length`, `split_on_word`
- [x] 1.2 Add `Default` impl for `InferenceConfig` with all default values
- [x] 1.3 Add `inference: InferenceConfig` field to `Config` struct
- [x] 1.4 Add validation for parameter ranges in `Config::validate()`

## 2. CLI Parameters

- [x] 2.1 Add CLI arguments to `Cli` struct in `src/cli.rs`: `--initial-prompt`, `--temperature`, `--suppress-non-speech`, `--no-speech-threshold`, `--max-segment-length`, `--split-on-word`
- [x] 2.2 Add corresponding fields to `CliOverrides` struct in `src/config.rs`
- [x] 2.3 Update `From<&Cli> for CliOverrides` to map CLI args to overrides

## 3. Config Merge Logic

- [x] 3.1 Update `Config::merge_with_cli()` to handle inference parameter overrides

## 4. Transcription Integration

- [x] 4.1 Update `transcribe_with_whisper()` in `src/transcription.rs` to apply all 6 inference parameters to `FullParams`

## 5. Default Config Generation

- [x] 5.1 Update `Config::to_yaml_string()` to include `inference:` section with all parameters and defaults
- [x] 5.2 Verify `transcriber init` generates complete default config

## 6. Documentation

- [x] 6.1 Update README.md or help text to document new CLI parameters
- [x] 6.2 Add comment in default config file explaining each parameter

## 7. Verification

- [x] 7.1 Run `cargo build` to verify compilation
- [x] 7.2 Run `cargo test` to verify existing tests pass
- [x] 7.3 Test `transcriber init` generates correct default config with inference section
- [x] 7.4 Manual test: run `transcriber -i video.mp4 --initial-prompt "测试" --temperature 0.2` and verify no errors

---

*Implementation complete. All 28 tasks verified and passing.*