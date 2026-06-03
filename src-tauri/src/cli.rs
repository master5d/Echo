use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug, Clone, Default)]
#[command(name = "echo", about = "Echo - Speech to Text")]
pub struct CliArgs {
    /// Start with the main window hidden
    #[arg(long)]
    pub start_hidden: bool,

    /// Disable the system tray icon
    #[arg(long)]
    pub no_tray: bool,

    /// Toggle transcription on/off (sent to running instance)
    #[arg(long)]
    pub toggle_transcription: bool,

    /// Toggle transcription with post-processing on/off (sent to running instance)
    #[arg(long)]
    pub toggle_post_process: bool,

    /// Cancel the current operation (sent to running instance)
    #[arg(long)]
    pub cancel: bool,

    /// Enable debug mode with verbose logging
    #[arg(long)]
    pub debug: bool,

    /// Path to an audio/video file for offline transcription
    #[arg(long, value_name = "FILE")]
    pub transcribe_file: Option<PathBuf>,

    /// Path to save the resulting transcript (plain text). Prints to stdout if omitted.
    #[arg(short, long, value_name = "OUTPUT")]
    pub output: Option<PathBuf>,

    /// Language for offline transcription: "auto" (default) or a code like "ru"/"en".
    /// "auto" enables RU/EN code-switch steering on Whisper models.
    #[arg(long, value_name = "LANG")]
    pub language: Option<String>,

    /// Model id to use for offline transcription (e.g. "turbo"). Defaults to the
    /// app's selected model. Use a Whisper model for best RU/EN mixing.
    #[arg(long, value_name = "MODEL")]
    pub model: Option<String>,

    /// Output format for offline transcription: plain (default), inline, srt, vtt, json, karaoke, speaker.
    /// `speaker` groups diarized turns as `[Speaker N] (M:SS - M:SS)` blocks.
    #[arg(long, value_name = "FORMAT")]
    pub format: Option<String>,

    /// Enable speaker diarization (label sections with Speaker 1/2/...).
    #[arg(long)]
    pub diarize: bool,

    /// Optional known speaker count hint for diarization (else auto-detected).
    #[arg(long, value_name = "N")]
    pub speakers: Option<usize>,
}
