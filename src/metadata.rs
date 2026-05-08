use crate::error::{AppError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// Auto-detected media metadata from ffprobe
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MediaMetadata {
    pub source: SourceInfo,
    pub video: Option<VideoInfo>,
    pub audio: Option<AudioInfo>,
    pub media_tags: HashMap<String, String>,
}

/// Container-level format information
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SourceInfo {
    pub filename: String,
    pub size_bytes: u64,
    pub format: String,
    pub bitrate: Option<u64>,
    pub duration_secs: f64,
}

/// Video stream information
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VideoInfo {
    pub codec: String,
    pub width: u32,
    pub height: u32,
    pub fps: Option<f64>,
    pub bitrate: Option<u64>,
}

/// Audio stream information
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AudioInfo {
    pub codec: String,
    pub sample_rate: u32,
    pub channels: u32,
    pub bitrate: Option<u64>,
}

/// Probe media file metadata via ffprobe
pub struct MediaProbe;

impl MediaProbe {
    /// Run ffprobe and parse metadata from the given media file.
    /// Silently returns minimal metadata if ffprobe fails (non-fatal).
    pub fn probe(path: &Path) -> Result<MediaMetadata> {
        let output = std::process::Command::new("ffprobe")
            .args([
                "-v",
                "quiet",
                "-print_format",
                "json",
                "-show_format",
                "-show_streams",
            ])
            .arg(path)
            .output()
            .map_err(|e| AppError::Probe {
                message: format!("Failed to run ffprobe: {}", e),
            })?;

        if !output.status.success() {
            return Ok(Self::minimal(path));
        }

        let raw: FfprobeOutput =
            serde_json::from_slice(&output.stdout).map_err(|e| AppError::Probe {
                message: format!("Failed to parse ffprobe output: {}", e),
            })?;

        Ok(Self::from_ffprobe(path, raw))
    }

    /// Build minimal metadata (fallback when ffprobe is unavailable or fails)
    pub fn minimal(path: &Path) -> MediaMetadata {
        let filename = path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();

        let ext = path
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();

        let size_bytes = std::fs::metadata(path).map(|m| m.len()).unwrap_or(0);

        MediaMetadata {
            source: SourceInfo {
                filename,
                size_bytes,
                format: ext,
                bitrate: None,
                duration_secs: 0.0,
            },
            video: None,
            audio: None,
            media_tags: HashMap::new(),
        }
    }

    fn from_ffprobe(path: &Path, raw: FfprobeOutput) -> MediaMetadata {
        let filename = path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();

        let source = SourceInfo {
            filename,
            size_bytes: raw.format.size.parse().unwrap_or(0),
            format: raw.format.format_name,
            bitrate: raw.format.bit_rate.and_then(|b| b.parse().ok()),
            duration_secs: raw.format.duration.parse().unwrap_or(0.0),
        };

        let mut video: Option<VideoInfo> = None;
        let mut audio: Option<AudioInfo> = None;
        let mut media_tags = HashMap::new();

        for stream in raw.streams {
            match stream.codec_type.as_str() {
                "video" => {
                    video = Some(VideoInfo {
                        codec: stream.codec_name,
                        width: stream.width.unwrap_or(0),
                        height: stream.height.unwrap_or(0),
                        fps: stream.r_frame_rate.as_ref().and_then(|r| parse_fraction(r)),
                        bitrate: stream.bit_rate.and_then(|b| b.parse().ok()),
                    });
                }
                "audio" => {
                    audio = Some(AudioInfo {
                        codec: stream.codec_name,
                        sample_rate: stream.sample_rate.and_then(|s| s.parse().ok()).unwrap_or(0),
                        channels: stream.channels.unwrap_or(0),
                        bitrate: stream.bit_rate.and_then(|b| b.parse().ok()),
                    });
                }
                _ => {}
            }
        }

        // Extract format-level tags
        if let Some(tags) = raw.format.tags {
            for (k, v) in tags {
                media_tags.insert(k, v);
            }
        }

        MediaMetadata {
            source,
            video,
            audio,
            media_tags,
        }
    }
}

/// Parse a fraction string like "30000/1001" into f64
fn parse_fraction(s: &str) -> Option<f64> {
    let parts: Vec<&str> = s.split('/').collect();
    match parts.len() {
        1 => parts[0].parse::<f64>().ok(),
        2 => {
            let num = parts[0].parse::<f64>().ok()?;
            let den = parts[1].parse::<f64>().ok()?;
            if den == 0.0 {
                None
            } else {
                Some(num / den)
            }
        }
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// Raw ffprobe JSON shape
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
struct FfprobeOutput {
    streams: Vec<FfprobeStream>,
    format: FfprobeFormat,
}

#[derive(Debug, Deserialize)]
struct FfprobeStream {
    codec_type: String,
    codec_name: String,
    #[serde(default)]
    width: Option<u32>,
    #[serde(default)]
    height: Option<u32>,
    #[serde(default)]
    r_frame_rate: Option<String>,
    #[serde(default)]
    bit_rate: Option<String>,
    #[serde(default)]
    sample_rate: Option<String>,
    #[serde(default)]
    channels: Option<u32>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct FfprobeFormat {
    filename: String,
    #[serde(default)]
    size: String,
    #[serde(default)]
    format_name: String,
    #[serde(default)]
    duration: String,
    #[serde(default)]
    bit_rate: Option<String>,
    #[serde(default)]
    tags: Option<HashMap<String, String>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_fraction_simple() {
        assert!((parse_fraction("30.0").unwrap() - 30.0).abs() < 0.001);
    }

    #[test]
    fn test_parse_fraction_rational() {
        let fps = parse_fraction("30000/1001").unwrap();
        assert!((fps - 29.97).abs() < 0.01);
    }

    #[test]
    fn test_parse_fraction_zero_denominator() {
        assert!(parse_fraction("1/0").is_none());
    }

    #[test]
    fn test_parse_fraction_invalid() {
        assert!(parse_fraction("not/valid").is_none());
    }

    #[test]
    fn test_minimal_metadata() {
        // Use a temp file so metadata is readable
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.mp4");
        std::fs::write(&path, b"fake content").unwrap();

        let meta = MediaProbe::minimal(&path);
        assert_eq!(meta.source.filename, "test.mp4");
        assert_eq!(meta.source.format, "mp4");
        assert!(meta.source.size_bytes > 0);
        assert!(meta.video.is_none());
        assert!(meta.audio.is_none());
    }

    #[test]
    fn test_from_ffprobe_parses_streams() {
        let json = serde_json::json!({
            "streams": [
                {
                    "codec_type": "video",
                    "codec_name": "h264",
                    "width": 1920,
                    "height": 1080,
                    "r_frame_rate": "30000/1001",
                    "bit_rate": "2000000"
                },
                {
                    "codec_type": "audio",
                    "codec_name": "aac",
                    "sample_rate": "48000",
                    "channels": 2,
                    "bit_rate": "128000"
                }
            ],
            "format": {
                "filename": "test.mp4",
                "size": "1234567",
                "format_name": "mp4",
                "duration": "120.5",
                "bit_rate": "2500000",
                "tags": {
                    "title": "My Video",
                    "artist": "Creator"
                }
            }
        });

        let raw: FfprobeOutput = serde_json::from_value(json).unwrap();
        let path = std::path::Path::new("test.mp4");
        let meta = MediaProbe::from_ffprobe(path, raw);

        assert_eq!(meta.source.filename, "test.mp4");
        assert_eq!(meta.source.size_bytes, 1234567);
        assert_eq!(meta.source.format, "mp4");
        assert_eq!(meta.source.bitrate, Some(2500000));
        assert!((meta.source.duration_secs - 120.5).abs() < 0.001);

        let v = meta.video.unwrap();
        assert_eq!(v.codec, "h264");
        assert_eq!(v.width, 1920);
        assert_eq!(v.height, 1080);
        assert!((v.fps.unwrap() - 29.97).abs() < 0.01);
        assert_eq!(v.bitrate, Some(2000000));

        let a = meta.audio.unwrap();
        assert_eq!(a.codec, "aac");
        assert_eq!(a.sample_rate, 48000);
        assert_eq!(a.channels, 2);
        assert_eq!(a.bitrate, Some(128000));

        assert_eq!(meta.media_tags.get("title"), Some(&"My Video".to_string()));
        assert_eq!(meta.media_tags.get("artist"), Some(&"Creator".to_string()));
    }

    #[test]
    fn test_from_ffprobe_no_tags() {
        let json = serde_json::json!({
            "streams": [],
            "format": {
                "filename": "test.mp4",
                "size": "0",
                "format_name": "mp4",
                "duration": "0"
            }
        });

        let raw: FfprobeOutput = serde_json::from_value(json).unwrap();
        let path = std::path::Path::new("test.mp4");
        let meta = MediaProbe::from_ffprobe(path, raw);

        assert_eq!(meta.source.filename, "test.mp4");
        assert!(meta.video.is_none());
        assert!(meta.audio.is_none());
        assert!(meta.media_tags.is_empty());
    }
}
