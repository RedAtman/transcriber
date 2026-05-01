use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(
    name = "transcriber",
    version = "0.1.0",
    about = "A fast video transcription tool powered by whisper.cpp",
    long_about = "transcriber extracts audio from video files and transcribes it to text.\n\
                   Supports GPU acceleration (Metal/Vulkan/CUDA) and multiple output formats.\n\n\
                   Examples:\n\
                     transcriber -i video.mp4\n\
                     transcriber -d ./videos -l zh -m medium\n\
                     transcriber init\n\
                     transcriber -i input.mp4 -o ./output --format srt"
)]
pub struct Cli {
    /// Input video file path
    #[arg(short = 'i', long = "input", value_name = "FILE")]
    pub input: Option<String>,

    /// Batch processing directory
    #[arg(
        short = 'd',
        long = "dir",
        value_name = "DIR",
        conflicts_with = "input"
    )]
    pub dir: Option<String>,

    /// Whisper model name
    #[arg(
        short = 'm',
        long = "model",
        value_name = "MODEL",
        help = "Model name: tiny/base/small/medium/large-v3-turbo"
    )]
    pub model: Option<String>,

    /// Language code
    #[arg(
        short = 'l',
        long = "language",
        value_name = "LANG",
        help = "Language code (ISO 639-1, e.g. zh/en/ja) or 'auto'"
    )]
    pub language: Option<String>,

    /// Output directory
    #[arg(short = 'o', long = "output", value_name = "DIR")]
    pub output: Option<String>,

    /// Custom config file path
    #[arg(long = "config", value_name = "FILE")]
    pub config: Option<String>,

    /// Skip existing transcript files
    #[arg(
        long = "skip-existing",
        help = "Skip files that already have transcript output"
    )]
    pub skip_existing: bool,

    /// Output format
    #[arg(
        long = "format",
        value_name = "FMT",
        help = "Output format: txt/srt/json/all (comma separated)"
    )]
    pub format: Option<String>,

    /// Number of CPU threads
    #[arg(
        long = "threads",
        value_name = "N",
        help = "Number of CPU threads (0 = auto)"
    )]
    pub threads: Option<u32>,

    /// GPU backend
    #[arg(
        long = "gpu",
        value_name = "BACKEND",
        help = "GPU backend: auto/metal/cuda/vulkan/cpu"
    )]
    pub gpu: Option<String>,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Generate default config file in current directory
    #[command(name = "init")]
    Init {
        /// Config file output path (default: ~/.config/transcriber/config.yaml)
        #[arg(long = "path", value_name = "FILE")]
        path: Option<String>,
    },
}

impl Cli {
    /// Check if input source is provided (file or directory)
    pub fn has_input_source(&self) -> bool {
        self.input.is_some() || self.dir.is_some()
    }

    /// Check if this is an init command
    pub fn is_init_command(&self) -> bool {
        matches!(self.command, Some(Commands::Init { .. }))
    }
}
