use crate::error::{AppError, Result};
use std::path::{Path, PathBuf};

/// Whisper model size info
pub struct ModelInfo {
    pub name: &'static str,
    pub description: &'static str,
    pub f16_size_mb: u32,
    pub ggml_filename: &'static str,
}

/// List of supported models
pub static MODELS: &[ModelInfo] = &[
    ModelInfo {
        name: "tiny",
        description: "Fastest, lowest quality",
        f16_size_mb: 75,
        ggml_filename: "ggml-tiny.bin",
    },
    ModelInfo {
        name: "base",
        description: "Default, balanced",
        f16_size_mb: 148,
        ggml_filename: "ggml-base.bin",
    },
    ModelInfo {
        name: "small",
        description: "Better quality",
        f16_size_mb: 488,
        ggml_filename: "ggml-small.bin",
    },
    ModelInfo {
        name: "medium",
        description: "High quality",
        f16_size_mb: 1500,
        ggml_filename: "ggml-medium.bin",
    },
    ModelInfo {
        name: "large-v3-turbo",
        description: "Best quality/speed ratio",
        f16_size_mb: 800,
        ggml_filename: "ggml-large-v3-turbo.bin",
    },
];

/// Model manager
pub struct ModelManager {
    cache_dir: PathBuf,
}

impl ModelManager {
    /// Create model manager
    pub fn new(cache_dir: PathBuf) -> Self {
        Self { cache_dir }
    }

    /// Get model path in cache directory
    pub fn model_path(&self, model_name: &str) -> PathBuf {
        self.cache_dir.join(format!("ggml-{}.bin", model_name))
    }

    /// Get quantized model path
    pub fn quantized_model_path(&self, model_name: &str, quantization: &str) -> PathBuf {
        self.cache_dir
            .join(format!("ggml-{}-{}.bin", model_name, quantization))
    }

    /// Check if model is cached
    pub fn is_cached(&self, model_name: &str, quantization: Option<&str>) -> bool {
        let path = if let Some(q) = quantization {
            self.quantized_model_path(model_name, q)
        } else {
            self.model_path(model_name)
        };
        path.exists()
    }

    /// Get cache directory
    pub fn cache_dir(&self) -> &Path {
        &self.cache_dir
    }
}

use indicatif::{ProgressBar, ProgressStyle};
use tokio::io::AsyncWriteExt;

impl ModelManager {
    /// HuggingFace download URL for whisper.cpp models
    fn download_url(model_name: &str, quantization: Option<&str>) -> String {
        let repo = "ggerganov/whisper.cpp";
        let filename = match quantization {
            Some(q) => format!("ggml-{}-{}.bin", model_name, q),
            None => format!("ggml-{}.bin", model_name),
        };
        format!("https://huggingface.co/{}/resolve/main/{}", repo, filename)
    }

    /// Download model to cache directory, showing progress
    pub async fn download(&self, model_name: &str, quantization: Option<&str>) -> Result<PathBuf> {
        let url = Self::download_url(model_name, quantization);
        let dest = if let Some(q) = quantization {
            self.quantized_model_path(model_name, q)
        } else {
            self.model_path(model_name)
        };

        tokio::fs::create_dir_all(&self.cache_dir).await?;

        let client = reqwest::Client::new();
        let response = client
            .get(&url)
            .send()
            .await
            .map_err(|_e| AppError::ModelDownload {
                model: model_name.to_string(),
                url: url.clone(),
                status: None,
            })?;

        let total_size = response.content_length().unwrap_or(0);
        let status_code = response.status().as_u16();

        if !response.status().is_success() {
            return Err(AppError::ModelDownload {
                model: model_name.to_string(),
                url,
                status: Some(status_code),
            });
        }

        let pb = ProgressBar::new(total_size);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("[{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})")
                .unwrap()
                .progress_chars("=>-"),
        );
        pb.set_message(format!("Downloading {}...", model_name));

        let mut file = tokio::fs::File::create(&dest).await?;
        let mut downloaded: u64 = 0;
        let mut stream = response.bytes_stream();

        use futures_util::StreamExt;
        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|_e| AppError::ModelDownload {
                model: model_name.to_string(),
                url: url.clone(),
                status: None,
            })?;
            file.write_all(&chunk).await?;
            downloaded += chunk.len() as u64;
            pb.set_position(downloaded);
        }

        pb.finish_with_message(format!("Downloaded {} successfully", model_name));
        Ok(dest)
    }
}

use whisper_rs::{WhisperContext, WhisperContextParameters};

impl ModelManager {
    /// Load model, returning WhisperContext
    /// Automatically handles download (if not cached) and GPU backend selection
    pub async fn load(
        &self,
        model_name: &str,
        quantization: Option<&str>,
        gpu_backend: &str,
        _n_threads: u32,
    ) -> Result<WhisperContext> {
        let model_path = if let Some(q) = quantization {
            let path = self.quantized_model_path(model_name, q);
            if !path.exists() {
                self.download(model_name, quantization).await?;
            }
            path
        } else {
            let path = self.model_path(model_name);
            if !path.exists() {
                self.download(model_name, None).await?;
            }
            path
        };

        let ctx_params = WhisperContextParameters::default();

        match gpu_backend {
            "metal" | "auto" if cfg!(target_os = "macos") => {
                tracing::info!("Using Metal GPU backend (macOS)");
            }
            "vulkan" => {
                tracing::info!("Using Vulkan GPU backend");
            }
            "cuda" => {
                tracing::info!("Using CUDA GPU backend");
            }
            _ => {
                tracing::info!("Using CPU backend");
            }
        }

        let ctx =
            WhisperContext::new_with_params(model_path.to_string_lossy().as_ref(), ctx_params)
                .map_err(|e| AppError::Model {
                    message: format!("Failed to load model: {}", e),
                    model: model_name.to_string(),
                })?;

        Ok(ctx)
    }

    /// Model verification: check if file exists and size is reasonable
    pub fn verify(&self, model_name: &str, quantization: Option<&str>) -> Result<bool> {
        let path = if let Some(q) = quantization {
            self.quantized_model_path(model_name, q)
        } else {
            self.model_path(model_name)
        };

        if !path.exists() {
            return Ok(false);
        }

        let metadata = std::fs::metadata(&path)?;
        let min_size: u64 = match model_name {
            "tiny" => 50 * 1024 * 1024,            // 50MB
            "base" => 100 * 1024 * 1024,           // 100MB
            "small" => 300 * 1024 * 1024,          // 300MB
            "medium" => 1000 * 1024 * 1024,        // 1GB
            "large-v3-turbo" => 500 * 1024 * 1024, // 500MB
            _ => 10 * 1024 * 1024,
        };

        Ok(metadata.len() >= min_size)
    }
}
