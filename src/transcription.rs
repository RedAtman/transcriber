use crate::audio::{AudioExtractor, file_stem, is_supported_video};
use crate::config::Config;
use crate::error::{AppError, Result};
use crate::model::ModelManager;
use crate::output::{self, Transcript, Segment, Word};
use indicatif::{ProgressBar, ProgressStyle, MultiProgress};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// Read PCM f32 samples from WAV file
fn load_wav_as_f32(path: &Path) -> Result<Vec<f32>> {
    let mut reader = hound::WavReader::open(path)
        .map_err(|e| AppError::Audio {
            message: format!("Failed to read WAV file: {}", e),
            source: None,
        })?;

    let spec = reader.spec();
    if spec.sample_rate != 16000 {
        return Err(AppError::Audio {
            message: format!("Expected 16kHz sample rate, got {}", spec.sample_rate),
            source: None,
        });
    }

    let samples: Vec<f32> = match spec.sample_format {
        hound::SampleFormat::Int => {
            reader.samples::<i16>()
                .map(|s| s.unwrap_or(0) as f32 / 32768.0)
                .collect()
        }
        hound::SampleFormat::Float => {
            reader.samples::<f32>()
                .map(|s| s.unwrap_or(0.0))
                .collect()
        }
    };

    Ok(samples)
}

use whisper_rs::{WhisperContext, FullParams, SamplingStrategy};
use chrono::Utc;

/// Core transcriber
pub struct Transcriber {
    config: Config,
}

impl Transcriber {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    /// Transcribe a single video file
    pub async fn transcribe_file(
        &self,
        video_path: &Path,
        model_mgr: &ModelManager,
        pb: Option<&ProgressBar>,
    ) -> Result<Transcript> {
        let video_path = video_path.to_path_buf();

        if let Some(pb) = pb {
            pb.set_message("Loading model...");
        }
        let ctx = model_mgr.load(
            &self.config.model.name,
            self.config.model.quantization.as_deref(),
            &self.config.performance.gpu,
            self.config.performance.threads,
        ).await?;

        if let Some(pb) = pb {
            pb.set_message("Extracting audio...");
        }
        let audio_extractor = AudioExtractor::new()?;
        let temp_dir = audio_extractor.create_temp_dir()?;
        let audio_path = temp_dir.path().join("audio.wav");
        audio_extractor.extract(&video_path, &audio_path).await?;

        if let Some(pb) = pb {
            pb.set_message("Transcribing...");
        }
        let audio_samples = load_wav_as_f32(&audio_path)?;

        // 4. Run transcription
        let transcript = transcribe_with_whisper(
            &ctx,
            &audio_samples,
            &video_path,
            &self.config,
            pb,
        )?;

        // 5. Cleanup temp files (TempDir drops automatically)
        drop(temp_dir);

        Ok(transcript)
    }
}

/// Transcribe using whisper-rs
fn transcribe_with_whisper(
    ctx: &WhisperContext,
    audio_samples: &[f32],
    video_path: &Path,
    config: &Config,
    pb: Option<&ProgressBar>,
) -> Result<Transcript> {
    let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 5 });

    let n_threads: i32 = if config.performance.threads == 0 {
        num_cpus::get() as i32
    } else {
        config.performance.threads as i32
    };
    params.set_n_threads(n_threads);

    if config.model.language == "auto" {
        params.set_language(Some("auto"));
    } else {
        params.set_language(Some(&config.model.language));
    }
    params.set_translate(false);

    let mut state = ctx.create_state().map_err(|e| AppError::Transcription {
        message: format!("Failed to create whisper state: {}", e),
        segment: None,
    })?;

    state.full(params, audio_samples)
        .map_err(|e| AppError::Transcription {
            message: format!("Whisper inference failed: {}", e),
            segment: None,
        })?;

    if config.model.language == "auto" {
        let lang_id = state.full_lang_id_from_state();
        if lang_id >= 0 {
            tracing::info!("Detected language ID: {}", lang_id);
        }
    }

    let n_segments = state.full_n_segments();
    let duration = if n_segments > 0 {
        if let Some(seg) = state.get_segment(n_segments - 1) {
            seg.end_timestamp() as f64 / 100.0
        } else {
            0.0
        }
    } else {
        0.0
    };

    let mut segments = Vec::with_capacity(n_segments as usize);
    for i in 0..n_segments {
        let segment = state.get_segment(i).ok_or_else(|| AppError::Transcription {
            message: format!("Failed to get segment {}", i),
            segment: Some(i as usize),
        })?;

        let text = segment.to_str()
            .map_err(|e| AppError::Transcription {
                message: format!("Failed to get segment text: {}", e),
                segment: Some(i as usize),
            })?.to_string();
        let t0 = segment.start_timestamp() as f64 / 100.0;
        let t1 = segment.end_timestamp() as f64 / 100.0;

        let n_tokens = segment.n_tokens();
        let mut words = Vec::new();
        for j in 0..n_tokens {
            if let Some(token) = segment.get_token(j) {
                let token_text = token.to_str()
                    .unwrap_or_default()
                    .to_string();
                let token_data = token.token_data();
                words.push(Word {
                    word: token_text,
                    start: token_data.t0 as f64 / 100.0,
                    end: token_data.t1 as f64 / 100.0,
                });
            }
        }

        segments.push(Segment {
            start: t0,
            end: t1,
            text,
            words,
        });

        if let Some(pb) = pb {
            let pct = ((i + 1) as f64 / n_segments as f64) * 100.0;
            pb.set_position(pct as u64);
        }
    }

    Ok(Transcript {
        file: video_path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string(),
        model: config.model.name.clone(),
        language: config.model.language.clone(),
        duration,
        segments,
        transcribed_at: Utc::now(),
    })
}

/// Batch transcription statistics
#[derive(Debug, Default)]
pub struct BatchStats {
    pub total: usize,
    pub success: usize,
    pub failed: usize,
    pub skipped: usize,
    pub total_duration_secs: f64,
}

impl Transcriber {
    /// Transcribe all video files in a directory
    pub async fn transcribe_directory(&self, dir_path: &Path, model_mgr: &ModelManager) -> Result<BatchStats> {
        // Collect all video files
        let mut video_files = Vec::new();
        collect_video_files(dir_path, &mut video_files)?;

        if video_files.is_empty() {
            return Ok(BatchStats::default());
        }

        let total = video_files.len();
        let mut stats = BatchStats {
            total,
            ..Default::default()
        };

        // Setup multi-progress bars
        let mp = MultiProgress::new();
        let overall_pb = mp.add(ProgressBar::new(total as u64));
        overall_pb.set_style(
            ProgressStyle::default_bar()
                .template("[{elapsed_precise}] [{bar:40.green/white}] {pos}/{len} files ({percent}%)")
                .unwrap()
                .progress_chars("=>-"),
        );
        overall_pb.set_message("Processing videos...");

        for video in &video_files {
            let stem = file_stem(video);

// Check if should skip
            if self.should_skip(video) {
                stats.skipped += 1;
                overall_pb.inc(1);
                continue;
            }

            // Per-file progress bar
            let file_pb = mp.add(ProgressBar::new(100));
            file_pb.set_style(
                ProgressStyle::default_bar()
                    .template("[{msg}] {spinner:.cyan} {bar:20.cyan/blue}")
                    .unwrap()
                    .progress_chars("=>-"),
            );
            file_pb.set_message(format!("Processing {}", stem));

            match self.transcribe_file(video, model_mgr, Some(&file_pb)).await {
                Ok(transcript) => {
                    // Write output files
                    if let Err(e) = self.write_output(&transcript, video) {
                        tracing::error!("Failed to write output for {:?}: {}", video, e);
                        stats.failed += 1;
                    } else {
                        stats.success += 1;
                        stats.total_duration_secs += transcript.duration;
                    }
                }
                Err(e) => {
                    tracing::error!("Failed to transcribe {:?}: {}", video, e);
                    stats.failed += 1;
                }
            }

            file_pb.finish_and_clear();
            overall_pb.inc(1);
        }

        overall_pb.finish_with_message(format!(
            "Done: {} success, {} failed, {} skipped",
            stats.success, stats.failed, stats.skipped
        ));

        Ok(stats)
    }
}

impl Transcriber {
    /// Check if this file should be skipped (output already exists)
    fn should_skip(&self, video_path: &Path) -> bool {
        if !self.config.output.skip_existing {
            return false;
        }

        let stem = file_stem(video_path);
        let output_dir = if self.config.output.directory == "./" {
            video_path.parent().unwrap_or(Path::new(".")).to_path_buf()
        } else {
            PathBuf::from(&self.config.output.directory)
        };

        // Check if any configured output formats already exist
        for fmt in &self.config.output.formats {
            let out_path = output::output_file_path(&stem, fmt, &output_dir);
            if out_path.exists() {
                return true;
            }
        }

        false
    }

    /// Write transcript results to output files
    pub fn write_output(&self, transcript: &Transcript, video_path: &Path) -> Result<()> {
        let stem = file_stem(video_path);
        let output_dir = if self.config.output.directory == "./" {
            video_path.parent().unwrap_or(Path::new(".")).to_path_buf()
        } else {
            PathBuf::from(&self.config.output.directory)
        };

        // Ensure output directory exists
        std::fs::create_dir_all(&output_dir)?;

        // Check if formats includes "all"
        let formats: Vec<&str> = if self.config.output.formats.iter().any(|f| f == "all") {
            vec!["txt", "srt", "json"]
        } else {
            self.config.output.formats.iter().map(|s| s.as_str()).collect()
        };

        for fmt in formats {
            let writer = output::get_format_writer(fmt).ok_or_else(|| AppError::Output {
                message: format!("Unknown output format: {}", fmt),
                path: None,
            })?;

            let content = writer.write(transcript)?;
            let out_path = output::output_file_path(&stem, fmt, &output_dir);

            // Write file
            std::fs::write(&out_path, content)?;
            tracing::info!("Wrote {:?}", out_path);
        }

        Ok(())
    }
}

impl Transcriber {
    /// Create a shared cancel flag
    pub fn cancel_flag() -> Arc<AtomicBool> {
        Arc::new(AtomicBool::new(false))
    }

    /// Set up Ctrl+C handler
    pub fn setup_signal_handler(cancel: Arc<AtomicBool>) {
        let cancel_clone = cancel.clone();
        ctrlc::set_handler(move || {
            tracing::warn!("Received Ctrl+C, finishing current segment...");
            cancel_clone.store(true, Ordering::SeqCst);
        })
        .expect("Failed to set Ctrl+C handler");
    }
}

/// Recursively collect all video files in a directory
fn collect_video_files(dir: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
    if !dir.is_dir() {
        return Err(AppError::Config {
            message: format!("Not a directory: {:?}", dir),
            field: None,
        });
    }

    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_video_files(&path, files)?;
        } else if is_supported_video(&path) {
            files.push(path);
        }
    }

    Ok(())
}
