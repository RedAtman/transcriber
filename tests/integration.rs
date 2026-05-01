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
