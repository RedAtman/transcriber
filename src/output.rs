use chrono::{DateTime, Utc};
use serde::Serialize;
use std::io::{BufWriter, Write};
use std::path::Path;

/// Word-level timestamp
#[derive(Debug, Clone, Serialize)]
pub struct Word {
    pub word: String,
    pub start: f64,
    pub end: f64,
}

/// Single text segment
#[derive(Debug, Clone, Serialize)]
pub struct Segment {
    pub start: f64,
    pub end: f64,
    pub text: String,
    pub words: Vec<Word>,
}

/// Complete transcript result
#[derive(Debug, Clone, Serialize)]
pub struct Transcript {
    pub file: String,
    pub model: String,
    pub language: String,
    pub duration: f64,
    pub segments: Vec<Segment>,
    pub transcribed_at: DateTime<Utc>,
}

/// An open output file stream for progressive segment-by-segment writing
pub struct StreamOutput {
    pub writer: BufWriter<std::fs::File>,
    pub format: String,
    pub segment_count: usize,
}

use crate::error::Result;

/// Output format writer trait
pub trait FormatWriter {
    fn extension(&self) -> &'static str;
    fn write(&self, transcript: &Transcript) -> Result<String>;
}

/// TXT format: one line of text per segment
pub struct TxtFormat;

impl FormatWriter for TxtFormat {
    fn extension(&self) -> &'static str {
        "txt"
    }

    fn write(&self, transcript: &Transcript) -> Result<String> {
        let mut output = String::new();
        for segment in &transcript.segments {
            output.push_str(&segment.text);
            output.push('\n');
        }
        Ok(output)
    }
}

/// SRT format: subtitle index + timestamp + text
pub struct SrtFormat;

impl FormatWriter for SrtFormat {
    fn extension(&self) -> &'static str {
        "srt"
    }

    fn write(&self, transcript: &Transcript) -> Result<String> {
        let mut output = String::new();
        for (i, segment) in transcript.segments.iter().enumerate() {
            let start = format_timestamp_srt(segment.start);
            let end = format_timestamp_srt(segment.end);
            output.push_str(&format!(
                "{}\n{} --> {}\n{}\n\n",
                i + 1,
                start,
                end,
                segment.text
            ));
        }
        Ok(output)
    }
}

/// Convert seconds to SRT timestamp format (HH:MM:SS,mmm)
fn format_timestamp_srt(seconds: f64) -> String {
    let total_ms = (seconds * 1000.0) as u64;
    let hours = total_ms / 3_600_000;
    let minutes = (total_ms % 3_600_000) / 60_000;
    let secs = (total_ms % 60_000) / 1000;
    let millis = total_ms % 1000;
    format!("{:02}:{:02}:{:02},{:03}", hours, minutes, secs, millis)
}

/// JSON format: structured data with metadata and word-level timing
pub struct JsonFormat;

impl FormatWriter for JsonFormat {
    fn extension(&self) -> &'static str {
        "json"
    }

    fn write(&self, transcript: &Transcript) -> Result<String> {
        serde_json::to_string_pretty(transcript).map_err(|e| crate::error::AppError::Output {
            message: format!("Failed to serialize JSON: {}", e),
            path: None,
        })
    }
}

/// Get FormatWriter by format name
pub fn get_format_writer(format: &str) -> Option<Box<dyn FormatWriter>> {
    match format {
        "txt" => Some(Box::new(TxtFormat)),
        "srt" => Some(Box::new(SrtFormat)),
        "json" => Some(Box::new(JsonFormat)),
        _ => None,
    }
}

/// Generate output file path
/// stem: input filename (without extension)
/// format: format name (txt/srt/json)
/// output_dir: output directory
pub fn output_file_path(
    stem: &str,
    format: &str,
    output_dir: &std::path::Path,
) -> std::path::PathBuf {
    let filename = format!("{}.transcript.{}", stem, format);
    output_dir.join(filename)
}

/// Open one output file per format for streaming writes
pub fn open_stream_outputs(
    stem: &str,
    formats: &[String],
    output_dir: &Path,
) -> Result<Vec<StreamOutput>> {
    let mut streams = Vec::new();
    for fmt in formats {
        let path = output_file_path(stem, fmt, output_dir);
        let file = std::fs::File::create(&path).map_err(|e| crate::error::AppError::Output {
            message: format!("Cannot create output file {:?}: {}", path, e),
            path: Some(path),
        })?;
        let mut writer = BufWriter::new(file);

        if fmt == "json" {
            writeln!(writer, "[").map_err(|e| crate::error::AppError::Output {
                message: format!("Failed to write JSON header: {}", e),
                path: None,
            })?;
        }

        streams.push(StreamOutput {
            writer,
            format: fmt.clone(),
            segment_count: 0,
        });
    }
    Ok(streams)
}

/// Append one transcribed segment to all open stream output files
pub fn append_segment_to_streams(streams: &mut [StreamOutput], segment: &Segment) -> Result<()> {
    for stream in streams.iter_mut() {
        match stream.format.as_str() {
            "txt" => {
                writeln!(stream.writer, "{}", segment.text).map_err(|e| {
                    crate::error::AppError::Output {
                        message: format!("Failed to write TXT segment: {}", e),
                        path: None,
                    }
                })?;
            }
            "srt" => {
                let start = format_timestamp_srt(segment.start);
                let end = format_timestamp_srt(segment.end);
                write!(
                    stream.writer,
                    "{}\n{} --> {}\n{}\n\n",
                    stream.segment_count + 1,
                    start,
                    end,
                    segment.text,
                )
                .map_err(|e| crate::error::AppError::Output {
                    message: format!("Failed to write SRT segment: {}", e),
                    path: None,
                })?;
            }
            "json" => {
                let seg_json =
                    serde_json::to_string(segment).map_err(|e| crate::error::AppError::Output {
                        message: format!("Failed to serialize JSON segment: {}", e),
                        path: None,
                    })?;
                if stream.segment_count > 0 {
                    write!(stream.writer, ",\n  {}", seg_json).map_err(|e| {
                        crate::error::AppError::Output {
                            message: format!("Failed to write JSON segment: {}", e),
                            path: None,
                        }
                    })?;
                } else {
                    write!(stream.writer, "  {}", seg_json).map_err(|e| {
                        crate::error::AppError::Output {
                            message: format!("Failed to write JSON segment: {}", e),
                            path: None,
                        }
                    })?;
                }
            }
            _ => unreachable!(),
        }
        // Flush after each segment so output is visible progressively
        stream
            .writer
            .flush()
            .map_err(|e| crate::error::AppError::Output {
                message: format!("Failed to flush output file: {}", e),
                path: None,
            })?;
        stream.segment_count += 1;
    }
    Ok(())
}

/// Append one segment (from whisper callback) to all open stream output files.
/// Unlike append_segment_to_streams, this takes raw data without word tokens,
/// allowing it to be called from within the whisper inference callback.
pub fn write_stream_callback_segment(
    streams: &mut [StreamOutput],
    start_secs: f64,
    end_secs: f64,
    text: &str,
) -> Result<()> {
    for stream in streams.iter_mut() {
        match stream.format.as_str() {
            "txt" => {
                writeln!(stream.writer, "{}", text).map_err(|e| {
                    crate::error::AppError::Output {
                        message: format!("Failed to write TXT segment: {}", e),
                        path: None,
                    }
                })?;
            }
            "srt" => {
                let start = format_timestamp_srt(start_secs);
                let end = format_timestamp_srt(end_secs);
                write!(
                    stream.writer,
                    "{}\n{} --> {}\n{}\n\n",
                    stream.segment_count + 1,
                    start,
                    end,
                    text,
                )
                .map_err(|e| crate::error::AppError::Output {
                    message: format!("Failed to write SRT segment: {}", e),
                    path: None,
                })?;
            }
            "json" => {
                let seg_json = serde_json::json!({
                    "start": start_secs,
                    "end": end_secs,
                    "text": text,
                    "words": []
                });
                let json_str = serde_json::to_string(&seg_json).map_err(|e| {
                    crate::error::AppError::Output {
                        message: format!("Failed to serialize JSON segment: {}", e),
                        path: None,
                    }
                })?;
                if stream.segment_count > 0 {
                    write!(stream.writer, ",\n  {}", json_str).map_err(|e| {
                        crate::error::AppError::Output {
                            message: format!("Failed to write JSON segment: {}", e),
                            path: None,
                        }
                    })?;
                } else {
                    write!(stream.writer, "  {}", json_str).map_err(|e| {
                        crate::error::AppError::Output {
                            message: format!("Failed to write JSON segment: {}", e),
                            path: None,
                        }
                    })?;
                }
            }
            _ => unreachable!(),
        }
        stream
            .writer
            .flush()
            .map_err(|e| crate::error::AppError::Output {
                message: format!("Failed to flush output file: {}", e),
                path: None,
            })?;
        stream.segment_count += 1;
    }
    Ok(())
}

/// Finalize all stream output files (write footers, flush)
pub fn finalize_stream_outputs(streams: Vec<StreamOutput>) -> Result<()> {
    for mut stream in streams {
        match stream.format.as_str() {
            "json" => {
                writeln!(stream.writer, "\n]").map_err(|e| crate::error::AppError::Output {
                    message: format!("Failed to write JSON footer: {}", e),
                    path: None,
                })?;
            }
            _ => {}
        }
        stream
            .writer
            .flush()
            .map_err(|e| crate::error::AppError::Output {
                message: format!("Failed to flush output file: {}", e),
                path: None,
            })?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_transcript() -> Transcript {
        Transcript {
            file: "test.mp4".to_string(),
            model: "base".to_string(),
            language: "zh".to_string(),
            duration: 10.0,
            segments: vec![
                Segment {
                    start: 0.0,
                    end: 3.5,
                    text: "This is the first segment of recognized text.".to_string(),
                    words: vec![],
                },
                Segment {
                    start: 3.5,
                    end: 7.2,
                    text: "This is the second segment of recognized text.".to_string(),
                    words: vec![],
                },
            ],
            transcribed_at: Utc::now(),
        }
    }

    #[test]
    fn test_txt_format() {
        let transcript = sample_transcript();
        let formatter = TxtFormat;
        let output = formatter.write(&transcript).unwrap();
        assert_eq!(output, "This is the first segment of recognized text.\nThis is the second segment of recognized text.\n");
    }

    #[test]
    fn test_srt_format() {
        let transcript = sample_transcript();
        let formatter = SrtFormat;
        let output = formatter.write(&transcript).unwrap();
        assert!(output.contains("00:00:00,000 --> 00:00:03,500"));
        assert!(output.contains("00:00:03,500 --> 00:00:07,200"));
        assert!(output.contains("1\n"));
        assert!(output.contains("2\n"));
    }

    #[test]
    fn test_json_format() {
        let transcript = sample_transcript();
        let formatter = JsonFormat;
        let output = formatter.write(&transcript).unwrap();
        assert!(output.contains("\"file\": \"test.mp4\""));
        assert!(output.contains("\"language\": \"zh\""));
    }

    #[test]
    fn test_format_timestamp_srt() {
        assert_eq!(format_timestamp_srt(0.0), "00:00:00,000");
        assert_eq!(format_timestamp_srt(3.5), "00:00:03,500");
        assert_eq!(format_timestamp_srt(125.0), "00:02:05,000");
        assert_eq!(format_timestamp_srt(3600.0), "01:00:00,000");
    }

    #[test]
    fn test_output_file_path() {
        let path = output_file_path("video", "srt", std::path::Path::new("/out"));
        assert_eq!(path, std::path::PathBuf::from("/out/video.transcript.srt"));
    }

    #[test]
    fn test_get_format_writer() {
        assert!(get_format_writer("txt").is_some());
        assert!(get_format_writer("srt").is_some());
        assert!(get_format_writer("json").is_some());
        assert!(get_format_writer("invalid").is_none());
    }

    #[test]
    fn test_streaming_writes_progressively() {
        let dir = std::env::temp_dir().join("transcriber-test-streaming");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        let formats = vec!["txt".to_string(), "json".to_string()];
        let mut streams = open_stream_outputs("test", &formats, &dir).unwrap();

        let seg1 = Segment {
            start: 0.0,
            end: 2.0,
            text: "Hello world".to_string(),
            words: vec![],
        };
        append_segment_to_streams(&mut streams, &seg1).unwrap();

        let txt_path = dir.join("test.transcript.txt");
        let txt_content = std::fs::read_to_string(&txt_path).unwrap();
        assert_eq!(
            txt_content, "Hello world\n",
            "TXT should have first segment after flush"
        );

        let json_path = dir.join("test.transcript.json");
        let json_content = std::fs::read_to_string(&json_path).unwrap();
        assert!(
            json_content.starts_with("["),
            "JSON should start with array open"
        );
        assert!(
            json_content.contains("Hello world"),
            "JSON should contain first segment"
        );

        let seg2 = Segment {
            start: 2.0,
            end: 5.0,
            text: "Second segment".to_string(),
            words: vec![],
        };
        append_segment_to_streams(&mut streams, &seg2).unwrap();

        let txt_content = std::fs::read_to_string(&txt_path).unwrap();
        assert_eq!(
            txt_content, "Hello world\nSecond segment\n",
            "TXT should have both segments"
        );

        let json_content = std::fs::read_to_string(&json_path).unwrap();
        assert!(
            json_content.contains("Second segment"),
            "JSON should contain second segment"
        );

        finalize_stream_outputs(streams).unwrap();

        let json_content = std::fs::read_to_string(&json_path).unwrap();
        assert!(
            json_content.ends_with("\n]\n") || json_content.ends_with("\n]\n"),
            "JSON should end with array close, got: {:?}",
            json_content.chars().last()
        );

        let _ = std::fs::remove_dir_all(&dir);
    }
}
