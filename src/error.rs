use std::path::PathBuf;

#[derive(Debug)]
pub enum AppError {
    Config {
        message: String,
        field: Option<String>,
    },
    Model {
        message: String,
        model: String,
    },
    Audio {
        message: String,
        source: Option<String>,
    },
    Transcription {
        message: String,
        segment: Option<usize>,
    },
    Output {
        message: String,
        path: Option<PathBuf>,
    },
    Io(std::io::Error),
    FfmpegNotFound,
    ModelDownload {
        model: String,
        url: String,
        status: Option<u16>,
    },
}

pub type Result<T> = std::result::Result<T, AppError>;

use std::fmt;

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::Config { message, field } => {
                if let Some(field) = field {
                    write!(f, "Configuration error in '{}': {}", field, message)
                } else {
                    write!(f, "Configuration error: {}", message)
                }
            }
            AppError::Model { message, model } => {
                write!(f, "Model '{}' error: {}", model, message)
            }
            AppError::Audio { message, source } => {
                if let Some(src) = source {
                    write!(f, "Audio extraction error ({}): {}", src, message)
                } else {
                    write!(f, "Audio extraction error: {}", message)
                }
            }
            AppError::Transcription { message, segment } => {
                if let Some(seg) = segment {
                    write!(f, "Transcription error at segment {}: {}", seg, message)
                } else {
                    write!(f, "Transcription error: {}", message)
                }
            }
            AppError::Output { message, path } => {
                if let Some(p) = path {
                    write!(f, "Output error for {:?}: {}", p, message)
                } else {
                    write!(f, "Output error: {}", message)
                }
            }
            AppError::Io(err) => write!(f, "I/O error: {}", err),
            AppError::FfmpegNotFound => {
                write!(
                    f,
                    "FFmpeg not found. Please install FFmpeg:\n  \
                     macOS: brew install ffmpeg\n  \
                     Ubuntu: sudo apt install ffmpeg\n  \
                     Or ensure ffmpeg is in your PATH."
                )
            }
            AppError::ModelDownload { model, url, status } => {
                if let Some(code) = status {
                    write!(
                        f,
                        "Failed to download model '{}' from {} (HTTP {})",
                        model, url, code
                    )
                } else {
                    write!(f, "Failed to download model '{}' from {}", model, url)
                }
            }
        }
    }
}

impl std::error::Error for AppError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            AppError::Io(err) => Some(err),
            _ => None,
        }
    }
}

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        AppError::Io(err)
    }
}

impl From<serde_yaml::Error> for AppError {
    fn from(err: serde_yaml::Error) -> Self {
        AppError::Config {
            message: err.to_string(),
            field: None,
        }
    }
}
