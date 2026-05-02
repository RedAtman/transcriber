# PROJECT KNOWLEDGE BASE

**Generated:** 2026-05-01
**Commit:** f2585e8
**Branch:** main

## OVERVIEW
Video/audio transcription CLI tool powered by whisper.cpp. Extracts audio via FFmpeg, transcribes with GPU acceleration (Metal/CUDA/Vulkan), outputs TXT/SRT/JSON.

## STRUCTURE
```
./src/
├── main.rs          # Entry point, async main, config loading
├── lib.rs           # Module exports
├── cli.rs           # Clap CLI args (Chinese comments)
├── config.rs        # Config struct, YAML loading, validation
├── audio.rs         # FFmpeg wrapper, audio extraction
├── model.rs         # ModelManager: download/load whisper.cpp models
├── transcription.rs # Core transcription logic, batch processing
├── output.rs        # FormatWriter trait, TXT/SRT/JSON writers
└── error.rs        # AppError enum, Result alias
```

## WHERE TO LOOK
| Task | Location | Notes |
|------|----------|-------|
| CLI args | `src/cli.rs` | Clap derive, Chinese comments |
| Config loading | `src/config.rs` | YAML → Config struct |
| FFmpeg integration | `src/audio.rs` | Requires ffmpeg in PATH |
| Model download | `src/model.rs` | HuggingFace whisper.cpp |
| Transcription | `src/transcription.rs` | Main business logic |
| Output formats | `src/output.rs` | Trait-based FormatWriter |
| Error handling | `src/error.rs` | Centralized AppError |

## CONVENTIONS
- **Comments**: Chinese (mixed with code, unusual for OSS)
- **Error handling**: `AppError` enum with `Result<T>` alias
- **Module pattern**: Flat structure, no nested modules
- **Config**: YAML with `#[serde(default)]` and Default impl for all structs
- **Async**: `tokio::main`, async audio extraction, sync batch processing

## ANTI-PATTERNS (THIS PROJECT)
- No unsafe blocks
- No TODO/FIXME/HACK comments
- No explicit anti-pattern rules

## UNIQUE STYLES
- Batch processing re-loads model per file (no pooling)
- TempDir auto-cleanup on drop
- CLI validation via `config.validate()` returns warnings, not errors
- Chinese comments in CLI (unusual for OSS library)

## COMMANDS
```bash
cargo build --release    # Optimized build with LTO
cargo test               # Integration tests in tests/
transcriber init         # Generate default config
transcriber -i video.mp4 # Single file transcription
```

## NOTES
- Requires FFmpeg in PATH (checked at AudioExtractor::new)
- GPU backend: metal (macOS), cuda, vulkan, or cpu fallback
- Model cache: ~/.cache/whisper by default
- Output naming: `{stem}.transcript.{format}`
- OpenSpec workflow active (specs in openspec/changes/archive/)
