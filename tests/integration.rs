use std::io::Write;

#[test]
fn test_config_loading_from_yaml() -> Result<(), Box<dyn std::error::Error>> {
    let yaml_content = r#"
model:
  name: medium
  language: zh
  quantization: q5_k

performance:
  threads: 4
  gpu: auto

output:
  formats: ["txt", "srt"]
  skip_existing: false

logging:
  level: debug
"#;

    let mut file = tempfile::NamedTempFile::new()?;
    file.write_all(yaml_content.as_bytes())?;

    let config = transcriber::config::Config::from_file(file.path())?;
    assert_eq!(config.model.name, "medium");
    assert_eq!(config.model.language, "zh");
    assert_eq!(config.model.quantization, Some("q5_k".to_string()));
    assert_eq!(config.performance.threads, 4);
    assert_eq!(config.performance.gpu, "auto");
    assert_eq!(config.output.formats, vec!["txt", "srt"]);
    assert!(!config.output.skip_existing);
    assert_eq!(config.logging.level, "debug");
    Ok(())
}

#[test]
fn test_config_loading_missing_file() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::path::Path::new("/tmp/nonexistent_config_42.yaml");
    let config = transcriber::config::Config::from_file(path)?;
    assert_eq!(config.model.name, "base");
    assert_eq!(config.model.language, "auto");
    Ok(())
}

#[test]
fn test_config_loading_partial() -> Result<(), Box<dyn std::error::Error>> {
    let yaml_content = "model:\n  name: small\n";

    let mut file = tempfile::NamedTempFile::new()?;
    file.write_all(yaml_content.as_bytes())?;

    let config = transcriber::config::Config::from_file(file.path())?;
    assert_eq!(config.model.name, "small");
    assert_eq!(config.model.language, "auto");
    assert!(config.output.skip_existing);
    assert_eq!(config.performance.threads, 0);
    Ok(())
}

#[test]
fn test_config_validation_invalid_model() {
    let mut config = transcriber::config::Config::default();
    config.model.name = "nonexistent".to_string();

    let errors = config.validate();
    assert!(errors.iter().any(|e| e.contains("Invalid model name")));
}

#[test]
fn test_config_validation_valid() {
    let config = transcriber::config::Config::default();
    let errors = config.validate();
    assert!(errors.is_empty() || errors.iter().all(|e| e.contains("exceeds")));
}

#[test]
fn test_config_validation_invalid_language() {
    let mut config = transcriber::config::Config::default();
    config.model.language = "invalid_lang".to_string();

    let errors = config.validate();
    assert!(errors.iter().any(|e| e.contains("Invalid language code")));
}

#[test]
fn test_config_validation_invalid_gpu() {
    let mut config = transcriber::config::Config::default();
    config.performance.gpu = "nonexistent_gpu".to_string();

    let errors = config.validate();
    assert!(errors.iter().any(|e| e.contains("Invalid GPU backend")));
}

#[test]
fn test_config_write_default_and_reload() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempfile::tempdir()?;
    let config_path = dir.path().join("config.yaml");

    transcriber::config::Config::write_default(&config_path)?;

    assert!(config_path.exists());

    let config = transcriber::config::Config::from_file(&config_path)?;
    assert_eq!(config.model.name, "base");
    assert_eq!(config.output.formats, vec!["txt"]);

    Ok(())
}

#[test]
fn test_config_validation_temperature_out_of_range() {
    let mut config = transcriber::config::Config::default();
    config.inference.temperature = 1.5;

    let errors = config.validate();
    assert!(errors.iter().any(|e| e.contains("temperature")));
}

#[test]
fn test_config_validation_invalid_quantization() {
    let mut config = transcriber::config::Config::default();
    config.model.quantization = Some("invalid_quant".to_string());

    let errors = config.validate();
    assert!(errors.iter().any(|e| e.contains("Invalid quantization")));
}

#[test]
fn test_output_formatter_unknown_format() {
    use transcriber::output::get_format_writer;

    assert!(get_format_writer("txt").is_some());
    assert!(get_format_writer("srt").is_some());
    assert!(get_format_writer("json").is_some());
    assert!(get_format_writer("unknown").is_none());
    assert!(get_format_writer("").is_none());
}

#[test]
fn test_output_file_path_generation() {
    use transcriber::output::output_file_path;

    let path = output_file_path("my_video", "srt", std::path::Path::new("/output"));
    assert_eq!(
        path,
        std::path::PathBuf::from("/output/my_video.transcript.srt")
    );

    let path = output_file_path("video", "json", std::path::Path::new("./"));
    assert!(path.to_str().unwrap().contains("video.transcript.json"));
}

// ============================================================================
// Integration Tests (10.3)
// ============================================================================

#[test]
fn test_audio_extractor_creation() {
    let extractor = transcriber::audio::AudioExtractor::new();
    assert!(extractor.is_ok());

    let extractor = extractor.unwrap();
    let info = extractor.ffmpeg_info();
    assert!(info.version.contains("ffmpeg") || info.version == "unknown");
}

#[test]
fn test_audio_extraction_integration() -> Result<(), Box<dyn std::error::Error>> {
    use tokio::runtime::Runtime;

    let dir = tempfile::tempdir()?;
    let input_path = dir.path().join("test_input.wav");
    let output_path = dir.path().join("test_output.wav");

    let ffmpeg = which::which("ffmpeg").expect("FFmpeg not found");
    let output = std::process::Command::new(&ffmpeg)
        .args([
            "-f",
            "lavfi",
            "-i",
            "anullsrc=r=16000:cl=mono",
            "-t",
            "1",
            "-y",
        ])
        .arg(&input_path)
        .output()?;

    if !output.status.success() {
        return Err(format!(
            "FFmpeg failed to create test audio: {:?}",
            String::from_utf8_lossy(&output.stderr)
        )
        .into());
    }

    let rt = Runtime::new()?;
    rt.block_on(async {
        let extractor = transcriber::audio::AudioExtractor::new()?;
        extractor.extract(&input_path, &output_path).await
    })?;

    assert!(output_path.exists());
    let metadata = std::fs::metadata(&output_path)?;
    assert!(metadata.len() > 0);

    Ok(())
}

#[test]
fn test_supported_video_extensions() {
    use transcriber::audio::{is_supported_video, SUPPORTED_VIDEO_EXTENSIONS};

    assert!(SUPPORTED_VIDEO_EXTENSIONS.contains(&"mp4"));
    assert!(SUPPORTED_VIDEO_EXTENSIONS.contains(&"mov"));
    assert!(SUPPORTED_VIDEO_EXTENSIONS.contains(&"avi"));

    assert!(is_supported_video(std::path::Path::new("video.mp4")));
    assert!(is_supported_video(std::path::Path::new("video.MP4")));
    assert!(is_supported_video(std::path::Path::new(
        "/path/to/video.mov"
    )));
    assert!(!is_supported_video(std::path::Path::new("document.pdf")));
    assert!(!is_supported_video(std::path::Path::new("audio.mp3")));
}

#[test]
fn test_cli_overrides_merge() {
    use transcriber::config::{CliOverrides, Config};

    let mut config = Config::default();
    assert_eq!(config.model.name, "base");

    let overrides = CliOverrides {
        model: Some("medium".to_string()),
        language: Some("zh".to_string()),
        threads: Some(8),
        ..Default::default()
    };

    config.merge_with_cli(overrides);

    assert_eq!(config.model.name, "medium");
    assert_eq!(config.model.language, "zh");
    assert_eq!(config.performance.threads, 8);
    assert_eq!(config.performance.gpu, "auto");
}

#[test]
fn test_md_format_output() -> Result<(), Box<dyn std::error::Error>> {
    use std::collections::HashMap;
    use transcriber::metadata::{AudioInfo, MediaMetadata, SourceInfo, VideoInfo};
    use transcriber::output::{FormatWriter, MdFormat, Segment, Transcript};

    let meta = MediaMetadata {
        source: SourceInfo {
            filename: "video.mp4".to_string(),
            size_bytes: 12345,
            format: "mp4".to_string(),
            bitrate: Some(2000000),
            duration_secs: 60.0,
            created_at: Some("2024-06-15T10:30:00.000000Z".to_string()),
            modified_at: None,
        },
        video: Some(VideoInfo {
            codec: "h264".to_string(),
            width: 1920,
            height: 1080,
            fps: Some(30.0),
            bitrate: Some(1500000),
        }),
        audio: Some(AudioInfo {
            codec: "aac".to_string(),
            sample_rate: 48000,
            channels: 2,
            bitrate: Some(128000),
        }),
        media_tags: HashMap::new(),
    };

    let mut user_meta = HashMap::new();
    user_meta.insert("location".to_string(), "Beijing".to_string());

    let transcript = Transcript {
        file: "video.mp4".to_string(),
        model: "base".to_string(),
        language: "zh".to_string(),
        duration: 60.0,
        segments: vec![Segment {
            start: 0.0,
            end: 3.5,
            text: "Hello world.".to_string(),
            words: vec![],
        }],
        transcribed_at: chrono::Utc::now(),
        media_metadata: Some(meta),
        user_metadata: user_meta,
    };

    let output = MdFormat.write(&transcript)?;

    assert!(
        output.starts_with("---\n"),
        "Should start with YAML front matter"
    );
    assert!(output.contains("filename: video.mp4"));
    assert!(output.contains("codec: h264"));
    assert!(output.contains("channels: 2"));
    assert!(output.contains("location: Beijing"));
    assert!(output.contains("---\n\n"), "Should end front matter");
    assert!(output.contains("Hello world."), "Should contain body text");
    assert!(
        !output.contains("00:00:00"),
        "Should not contain timestamps"
    );

    Ok(())
}

#[test]
fn test_meta_parsing() {
    use clap::Parser;
    use transcriber::cli::Cli;
    use transcriber::config::CliOverrides;

    let cli = Cli::parse_from([
        "transcriber",
        "-i",
        "test.mp4",
        "--meta",
        "key1=value1",
        "--meta",
        "key2=value2",
    ]);
    let overrides = CliOverrides::from(&cli);
    assert_eq!(
        overrides.user_metadata.get("key1"),
        Some(&"value1".to_string())
    );
    assert_eq!(
        overrides.user_metadata.get("key2"),
        Some(&"value2".to_string())
    );
    assert_eq!(overrides.user_metadata.len(), 2);
}

#[test]
fn test_meta_parsing_no_value() {
    use clap::Parser;
    use transcriber::cli::Cli;
    use transcriber::config::CliOverrides;

    let cli = Cli::parse_from(["transcriber", "-i", "test.mp4", "--meta", "flag"]);
    let overrides = CliOverrides::from(&cli);
    assert_eq!(overrides.user_metadata.get("flag"), Some(&"".to_string()));
}

#[test]
fn test_config_user_metadata_merge() {
    use transcriber::config::Config;

    let mut config = Config::default();
    assert!(config.user_metadata.is_empty());

    let overrides = transcriber::config::CliOverrides {
        user_metadata: [("key1".into(), "val1".into())].into(),
        ..Default::default()
    };
    config.merge_with_cli(overrides);
    assert_eq!(config.user_metadata.get("key1"), Some(&"val1".to_string()));
}

#[test]
fn test_md_null_stripping() -> Result<(), Box<dyn std::error::Error>> {
    use std::collections::HashMap;
    use transcriber::metadata::{AudioInfo, MediaMetadata, SourceInfo, VideoInfo};
    use transcriber::output::{FormatWriter, MdFormat, Segment, Transcript};

    // All optional fields set to None — they should not appear in YAML
    // All optional fields set to None — they should not appear in YAML
    let meta = MediaMetadata {
        source: SourceInfo {
            filename: "video.mp4".to_string(),
            size_bytes: 12345,
            format: "mp4".to_string(),
            bitrate: None,
            duration_secs: 60.0,
            created_at: None,
            modified_at: None,
        },
        video: Some(VideoInfo {
            codec: "h264".to_string(),
            width: 1920,
            height: 1080,
            fps: None,
            bitrate: None,
        }),
        audio: Some(AudioInfo {
            codec: "aac".to_string(),
            sample_rate: 48000,
            channels: 2,
            bitrate: None,
        }),
        media_tags: HashMap::new(),
    };

    let transcript = Transcript {
        file: "video.mp4".to_string(),
        model: "base".to_string(),
        language: "en".to_string(),
        duration: 60.0,
        segments: vec![Segment {
            start: 0.0,
            end: 3.5,
            text: "Test content.".to_string(),
            words: vec![],
        }],
        transcribed_at: chrono::Utc::now(),
        media_metadata: Some(meta),
        user_metadata: HashMap::new(),
    };

    let output = MdFormat.write(&transcript)?;

    // Should NOT contain null fields
    assert!(!output.contains("fps:"), "null fps should be stripped");
    assert!(
        !output.contains("bitrate:"),
        "null bitrate should be stripped"
    );

    // Should still contain non-null fields
    assert!(output.contains("filename: video.mp4"));
    assert!(output.contains("codec: h264"));
    assert!(output.contains("channels: 2"));
    assert!(output.contains("Test content."));

    Ok(())
}

#[test]
fn test_error_type_display() {
    use transcriber::error::AppError;

    let err = AppError::FfmpegNotFound;
    let display = format!("{}", err);
    assert!(display.contains("FFmpeg not found"));

    let err = AppError::Config {
        message: "test error".to_string(),
        field: Some("model".to_string()),
    };
    let display = format!("{}", err);
    assert!(display.contains("test error"));
    assert!(display.contains("model"));
}
