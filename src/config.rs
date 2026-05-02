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
pub struct InferenceConfig {
    /// Initial prompt for decoder context
    pub initial_prompt: String,
    /// Sampling temperature (0.0 = greedy/deterministic, higher = more random)
    pub temperature: f32,
    /// Suppress non-speech tokens (cough, background noise, etc.)
    pub suppress_non_speech: bool,
    /// No-speech detection threshold (0.0-1.0)
    pub no_speech_threshold: f32,
    /// Maximum segment length in characters (0 = no limit)
    pub max_segment_length: u32,
    /// Split timestamps on word boundaries instead of characters
    pub split_on_word: bool,
}

impl Default for InferenceConfig {
    fn default() -> Self {
        Self {
            initial_prompt: String::new(),
            temperature: 0.0,
            suppress_non_speech: false,
            no_speech_threshold: 0.6,
            max_segment_length: 0,
            split_on_word: false,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub model: ModelConfig,
    pub audio: AudioConfig,
    pub performance: PerformanceConfig,
    pub output: OutputConfig,
    pub logging: LoggingConfig,
    pub cache: CacheConfig,
    pub inference: InferenceConfig,
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
        std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".cache").join("whisper"))
    }

    /// Load config from file, returns defaults if file doesn't exist
    pub fn from_file(path: &std::path::Path) -> Result<Self> {
        if !path.exists() {
            return Ok(Config::default());
        }
        let contents = std::fs::read_to_string(path).map_err(|e| AppError::Config {
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
    pub initial_prompt: Option<String>,
    pub temperature: Option<f32>,
    pub suppress_non_speech: Option<bool>,
    pub no_speech_threshold: Option<f32>,
    pub max_segment_length: Option<u32>,
    pub split_on_word: Option<bool>,
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
        if let Some(initial_prompt) = overrides.initial_prompt {
            self.inference.initial_prompt = initial_prompt;
        }
        if let Some(temperature) = overrides.temperature {
            self.inference.temperature = temperature;
        }
        if let Some(suppress_non_speech) = overrides.suppress_non_speech {
            self.inference.suppress_non_speech = suppress_non_speech;
        }
        if let Some(no_speech_threshold) = overrides.no_speech_threshold {
            self.inference.no_speech_threshold = no_speech_threshold;
        }
        if let Some(max_segment_length) = overrides.max_segment_length {
            self.inference.max_segment_length = max_segment_length;
        }
        if let Some(split_on_word) = overrides.split_on_word {
            self.inference.split_on_word = split_on_word;
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

pub static VALID_MODELS: &[&str] = &["tiny", "base", "small", "medium", "large-v3-turbo"];

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

        // Validate inference parameters
        if self.inference.temperature < 0.0 || self.inference.temperature > 1.0 {
            errors.push(format!(
                "Invalid temperature '{}'. Must be between 0.0 and 1.0.",
                self.inference.temperature
            ));
        }

        if self.inference.no_speech_threshold < 0.0 || self.inference.no_speech_threshold > 1.0 {
            errors.push(format!(
                "Invalid no_speech_threshold '{}'. Must be between 0.0 and 1.0.",
                self.inference.no_speech_threshold
            ));
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
            initial_prompt: cli.initial_prompt.clone(),
            temperature: cli.temperature,
            suppress_non_speech: if cli.suppress_non_speech {
                Some(true)
            } else {
                None
            },
            no_speech_threshold: cli.no_speech_threshold,
            max_segment_length: cli.max_segment_length,
            split_on_word: if cli.split_on_word {
                Some(true)
            } else {
                None
            },
        }
    }
}
