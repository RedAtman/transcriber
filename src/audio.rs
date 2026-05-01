use crate::error::{AppError, Result};
use std::path::{Path, PathBuf};
use std::process::Command;

/// FFmpeg detection result
#[derive(Debug, Clone)]
pub struct FfmpegInfo {
    pub path: PathBuf,
    pub version: String,
}

/// Audio extractor
pub struct AudioExtractor {
    ffmpeg: FfmpegInfo,
}

impl AudioExtractor {
    /// Detect system FFmpeg, returns error if not found
    pub fn new() -> Result<Self> {
        let ffmpeg = Self::find_ffmpeg().ok_or(AppError::FfmpegNotFound)?;
        Ok(Self { ffmpeg })
    }

    /// Find ffmpeg in PATH
    fn find_ffmpeg() -> Option<FfmpegInfo> {
        let path = which::which("ffmpeg").ok()?;
        let version = Command::new(&path)
            .arg("-version")
            .output()
            .ok()
            .and_then(|o| {
                if o.status.success() {
                    String::from_utf8(o.stdout)
                        .ok()
                        .and_then(|s| s.lines().next().map(|l| l.to_string()))
                } else {
                    None
                }
            })
            .unwrap_or_else(|| "unknown".to_string());

        Some(FfmpegInfo { path, version })
    }
}

use tokio::process::Command as TokioCommand;

impl AudioExtractor {
    /// Extract audio to WAV file and return the path
    /// Output: 16kHz mono PCM WAV
    pub async fn extract(&self, video_path: &Path, output_path: &Path) -> Result<()> {
        let status = TokioCommand::new(&self.ffmpeg.path)
            .arg("-i")
            .arg(video_path)
            .arg("-vn")
            .arg("-acodec")
            .arg("pcm_s16le")
            .arg("-ar")
            .arg("16000")
            .arg("-ac")
            .arg("1")
            .arg("-y")
            .arg(output_path)
            .status()
            .await
            .map_err(|e| AppError::Audio {
                message: format!("Failed to execute FFmpeg: {}", e),
                source: Some(e.to_string()),
            })?;

        if !status.success() {
            return Err(AppError::Audio {
                message: format!("FFmpeg exited with code: {:?}", status.code()),
                source: None,
            });
        }

        if !output_path.exists() || output_path.metadata()?.len() == 0 {
            return Err(AppError::Audio {
                message: "FFmpeg produced empty output".to_string(),
                source: None,
            });
        }

        Ok(())
    }

    /// Get FFmpeg info
    pub fn ffmpeg_info(&self) -> &FfmpegInfo {
        &self.ffmpeg
    }
}

use tempfile::TempDir;

impl AudioExtractor {
    /// Create temporary directory for intermediate files
    pub fn create_temp_dir(&self) -> Result<TempDir> {
        TempDir::new().map_err(|e| AppError::Audio {
            message: format!("Failed to create temp directory: {}", e),
            source: None,
        })
    }
}

/// Supported video file extensions
pub const SUPPORTED_VIDEO_EXTENSIONS: &[&str] = &["mp4", "mov", "avi", "mkv", "webm", "m4v"];

/// Check if file extension is supported
pub fn is_supported_video(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| SUPPORTED_VIDEO_EXTENSIONS.contains(&e.to_lowercase().as_str()))
        .unwrap_or(false)
}

/// Get file stem (without extension)
pub fn file_stem(path: &Path) -> String {
    path.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown")
        .to_string()
}
