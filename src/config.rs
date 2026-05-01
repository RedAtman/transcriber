use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ModelConfig {
    pub name: String,
    pub language: String,
    pub quantization: Option<String>,
}

impl Default for ModelConfig {
    fn default() -> Self {
        Self {
            name: "base".to_string(),
            language: "auto".to_string(),
            quantization: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AudioConfig {
    pub sample_rate: u32,
    pub channels: u32,
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            sample_rate: 16000,
            channels: 1,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct PerformanceConfig {
    pub threads: u32,
    pub gpu: String,
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            threads: 0,
            gpu: "auto".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct OutputConfig {
    pub formats: Vec<String>,
    pub directory: String,
    pub skip_existing: bool,
}

impl Default for OutputConfig {
    fn default() -> Self {
        Self {
            formats: vec!["txt".to_string()],
            directory: "./".to_string(),
            skip_existing: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct LoggingConfig {
    pub level: String,
    pub file: String,
    pub colors: bool,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            file: String::new(),
            colors: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct CacheConfig {
    pub directory: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub model: ModelConfig,
    pub audio: AudioConfig,
    pub performance: PerformanceConfig,
    pub output: OutputConfig,
    pub logging: LoggingConfig,
    pub cache: CacheConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            model: ModelConfig::default(),
            audio: AudioConfig::default(),
            performance: PerformanceConfig::default(),
            output: OutputConfig::default(),
            logging: LoggingConfig::default(),
            cache: CacheConfig::default(),
        }
    }
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            directory: "~/.cache/whisper".to_string(),
        }
    }
}

use crate::error::{AppError, Result};
use std::path::PathBuf;

impl Config {
    /// Default config path: ~/.config/transcriber/config.yaml
    pub fn default_config_path() -> Option<PathBuf> {
        std::env::var_os("HOME").map(|home| {
            PathBuf::from(home)
                .join(".config")
                .join("transcriber")
                .join("config.yaml")
        })
    }

    /// Default cache directory: ~/.cache/whisper
    pub fn default_cache_dir() -> Option<PathBuf> {
        std::env::var_os("HOME").map(|home| {
            PathBuf::from(home)
                .join(".cache")
                .join("whisper")
        })
    }

    /// Load config from file, returns defaults if file doesn't exist
    pub fn from_file(path: &std::path::Path) -> Result<Self> {
        if !path.exists() {
            return Ok(Config::default());
        }
        let contents = std::fs::read_to_string(path)
            .map_err(|e| AppError::Config {
                message: format!("Cannot read config file {:?}: {}", path, e),
                field: Some("file".to_string()),
            })?;
        let config: Config = serde_yaml::from_str(&contents)?;
        Ok(config)
    }
}

/// CLI parameter overrides, parsed from CLI arguments
#[derive(Debug, Default)]
pub struct CliOverrides {
    pub model: Option<String>,
    pub language: Option<String>,
    pub output_dir: Option<String>,
    pub config_path: Option<String>,
    pub format: Option<String>,
    pub threads: Option<u32>,
    pub gpu: Option<String>,
    pub skip_existing: Option<bool>,
}

impl Config {
    /// Merge CLI parameter overrides into config (CLI values take priority)
    pub fn merge_with_cli(&mut self, overrides: CliOverrides) {
        if let Some(model) = overrides.model {
            self.model.name = model;
        }
        if let Some(language) = overrides.language {
            self.model.language = language;
        }
        if let Some(dir) = overrides.output_dir {
            self.output.directory = dir;
        }
        if let Some(format) = overrides.format {
            self.output.formats = if format == "all" {
                vec!["txt".to_string(), "srt".to_string(), "json".to_string()]
            } else {
                format.split(',').map(|s| s.trim().to_string()).collect()
            };
        }
        if let Some(threads) = overrides.threads {
            self.performance.threads = threads;
        }
        if let Some(gpu) = overrides.gpu {
            self.performance.gpu = gpu;
        }
        if let Some(skip) = overrides.skip_existing {
            self.output.skip_existing = skip;
        }
        if self.cache.directory.starts_with('~') {
            if let Some(home) = std::env::var_os("HOME") {
                self.cache.directory = std::path::Path::new(&home)
                    .join(&self.cache.directory[2..])
                    .to_string_lossy()
                    .to_string();
            }
        }
    }
}

pub static VALID_MODELS: &[&str] = &[
    "tiny", "base", "small", "medium", "large-v3-turbo",
];

pub static VALID_QUANTIZATIONS: &[&str] = &["q4_k", "q5_k", "q6_k", "q8_0"];

impl Config {
    /// Validate config values, returns list of errors
    pub fn validate(&self) -> Vec<String> {
        let mut errors = Vec::new();

        if !VALID_MODELS.contains(&self.model.name.as_str()) {
            errors.push(format!(
                "Invalid model name '{}'. Valid options: {}",
                self.model.name,
                VALID_MODELS.join(", ")
            ));
        }

        if let Some(ref q) = self.model.quantization {
            if !VALID_QUANTIZATIONS.contains(&q.as_str()) {
                errors.push(format!(
                    "Invalid quantization '{}'. Valid options: {}",
                    q,
                    VALID_QUANTIZATIONS.join(", ")
                ));
            }
        }

        if self.model.language != "auto" && self.model.language.len() != 2 {
            errors.push(format!(
                "Invalid language code '{}'. Use ISO 639-1 (e.g. 'zh', 'en') or 'auto'.",
                self.model.language
            ));
        }

        let available_cores = num_cpus::get() as u32;
        if self.performance.threads > available_cores * 2 {
            errors.push(format!(
                "Thread count {} exceeds system capacity ({} cores). Using {} threads.",
                self.performance.threads, available_cores, available_cores
            ));
        }

        let valid_gpu = ["auto", "metal", "cuda", "vulkan", "cpu"];
        if !valid_gpu.contains(&self.performance.gpu.as_str()) {
            errors.push(format!(
                "Invalid GPU backend '{}'. Valid options: {}",
                self.performance.gpu,
                valid_gpu.join(", ")
            ));
        }

        // Validate output formats
        let valid_formats = ["txt", "srt", "json", "all"];
        for fmt in &self.output.formats {
            if !valid_formats.contains(&fmt.as_str()) {
                errors.push(format!(
                    "Invalid output format '{}'. Valid options: {}",
                    fmt,
                    valid_formats.join(", ")
                ));
            }
        }

        errors
    }
}

impl Config {
    /// Generate YAML string for default config
    pub fn to_yaml_string(&self) -> Result<String> {
        serde_yaml::to_string(self).map_err(|e| AppError::Config {
            message: format!("Failed to serialize default config: {}", e),
            field: None,
        })
    }

    /// Write default config file to specified path
    pub fn write_default(path: &std::path::Path) -> Result<()> {
        let config = Config::default();
        let yaml = config.to_yaml_string()?;

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        std::fs::write(path, yaml)?;
        Ok(())
    }
}

impl From<&crate::cli::Cli> for CliOverrides {
    fn from(cli: &crate::cli::Cli) -> Self {
        CliOverrides {
            model: cli.model.clone(),
            language: cli.language.clone(),
            output_dir: cli.output.clone(),
            config_path: cli.config.clone(),
            format: cli.format.clone(),
            threads: cli.threads,
            gpu: cli.gpu.clone(),
            skip_existing: if cli.skip_existing { Some(true) } else { None },
        }
    }
}
