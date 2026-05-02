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
