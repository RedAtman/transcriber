use clap::Parser;
use std::path::{Path, PathBuf};
use tracing_subscriber::EnvFilter;
use transcriber::cli::{Cli, Commands};
use transcriber::config::{Config, CliOverrides};
use transcriber::error::{AppError, Result};
use transcriber::model::ModelManager;
use transcriber::audio::AudioExtractor;
use transcriber::transcription::Transcriber;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let cli = Cli::parse();

    let result = match &cli.command {
        Some(Commands::Init { path }) => handle_init(path.as_deref()),
        _ => handle_transcribe(&cli).await,
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn handle_init(custom_path: Option<&str>) -> Result<()> {
    let config_path = match custom_path {
        Some(path) => PathBuf::from(path),
        None => Config::default_config_path()
            .ok_or_else(|| AppError::Config {
                message: "Cannot determine config directory. Use --path to specify.".to_string(),
                field: None,
            })?,
    };

    if config_path.exists() {
        tracing::warn!("Config file already exists at {:?}", config_path);
        tracing::warn!("Delete it first or specify a different path with --path.");
        return Ok(());
    }

    Config::write_default(&config_path)?;
    println!("Created default config at {:?}", config_path);
    println!("Edit this file to customize settings, then run transcriber -i <video>");
    Ok(())
}

async fn handle_transcribe(cli: &Cli) -> Result<()> {
    let config = load_config(cli)?;

    let warnings = config.validate();
    for warning in &warnings {
        tracing::warn!("Config: {}", warning);
    }

    let cancel = Transcriber::cancel_flag();
    Transcriber::setup_signal_handler(cancel);

    let cache_dir = resolve_cache_dir(&config);
    let model_mgr = ModelManager::new(cache_dir);

    let _ffmpeg = AudioExtractor::new()?;

    let transcriber = Transcriber::new(config);

    if let Some(input) = &cli.input {
        let video_path = Path::new(input);
        if !video_path.exists() {
            return Err(AppError::Config {
                message: format!("File not found: {}", input),
                field: Some("input".to_string()),
            });
        }

        println!("Transcribing: {}", input);
        let transcript = transcriber.transcribe_file(video_path, &model_mgr, None).await?;
        transcriber.write_output(&transcript, video_path)?;

        println!(
            "Done! {} segments, {:.1}s audio",
            transcript.segments.len(),
            transcript.duration
        );
    } else if let Some(dir) = &cli.dir {
        let dir_path = Path::new(dir);
        if !dir_path.is_dir() {
            return Err(AppError::Config {
                message: format!("Directory not found: {}", dir),
                field: Some("dir".to_string()),
            });
        }

        println!("Batch transcribing: {}", dir);
        let stats = transcriber.transcribe_directory(dir_path, &model_mgr).await?;
        println!(
            "Batch complete: {}/{} success, {} failed, {} skipped ({:.1}s audio)",
            stats.success, stats.total, stats.failed, stats.skipped, stats.total_duration_secs
        );
    } else {
        println!("No input provided. Use -i <file> or -d <directory>.");
        println!("Run 'transcriber --help' for usage.");
    }

    Ok(())
}

fn load_config(cli: &Cli) -> Result<Config> {
    let config_path = if let Some(ref path) = cli.config {
        PathBuf::from(path)
    } else {
        Config::default_config_path()
            .unwrap_or_else(|| Path::new("config.yaml").to_path_buf())
    };

    let mut config = if config_path.exists() {
        tracing::info!("Loading config from {:?}", config_path);
        Config::from_file(&config_path)?
    } else {
        tracing::info!("No config file found, using defaults");
        Config::default()
    };

    let overrides = CliOverrides::from(cli);
    config.merge_with_cli(overrides);

    Ok(config)
}

fn resolve_cache_dir(config: &Config) -> PathBuf {
    let dir = config.cache.directory.clone();
    if dir.starts_with('~') {
        if let Some(home) = std::env::var_os("HOME") {
            return Path::new(&home).join(&dir[2..]);
        }
    }
    if dir.is_empty() {
        Config::default_cache_dir()
            .unwrap_or_else(|| Path::new("~/.cache/whisper").to_path_buf())
    } else {
        PathBuf::from(&dir)
    }
}
