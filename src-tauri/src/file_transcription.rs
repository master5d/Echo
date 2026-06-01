//! Shared offline file-transcription core used by both the CLI
//! (`run_cli_transcription`) and the GUI Tauri command. Extracts audio via
//! ffmpeg, runs the engine for full details (text + segments), and — when
//! requested — runs speaker diarization and attaches the turns.

use crate::managers::model::{EngineType, ModelManager};
use crate::managers::transcription::{TranscriptionDetails, TranscriptionManager};
use crate::settings::{get_settings, write_settings};
use anyhow::{Context, Result};
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::Arc;
use tauri::{AppHandle, Manager};

/// Sample rate the engine and diarizer both expect (ffmpeg downmixes to this).
pub const TARGET_SAMPLE_RATE: u32 = 16_000;

/// Transcribe a file to full details. `diarize`/`speaker_hint` are accepted now
/// but only take effect once `diarization::diarize` is implemented (Phase 2);
/// until then a warning is emitted and speakers stay `None`.
pub fn transcribe_file_detailed(
    app_handle: &AppHandle,
    input: &Path,
    language: Option<&str>,
    model: Option<&str>,
    diarize: bool,
    speaker_hint: Option<usize>,
) -> Result<TranscriptionDetails> {
    let input_str = input.to_str().context("Input path is not valid UTF-8")?;

    // ffmpeg guard — a clear message beats a cryptic spawn failure.
    let ffmpeg_ok = Command::new("ffmpeg")
        .arg("-version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false);
    if !ffmpeg_ok {
        anyhow::bail!(
            "ffmpeg not found on PATH. Install it (e.g. `winget install Gyan.FFmpeg`) and retry."
        );
    }

    // Resolve effective model/language and warn if the model can't code-switch RU/EN.
    let base_settings = get_settings(app_handle);
    let effective_model = model
        .map(|s| s.to_string())
        .unwrap_or_else(|| base_settings.selected_model.clone());
    let effective_language = language
        .map(|s| s.to_string())
        .unwrap_or_else(|| base_settings.selected_language.clone());

    let model_mgr = app_handle.state::<Arc<ModelManager>>();
    if let Some(info) = model_mgr.get_model_info(&effective_model) {
        if !matches!(info.engine_type, EngineType::Whisper) {
            eprintln!(
                "[!] Model '{}' is {:?} (single-script). For Russian/English code-switching, \
use a Whisper model — e.g. `--model turbo`.",
                effective_model, info.engine_type
            );
        }
    }

    // Temporarily steer settings (language + model drive the bilingual prompt and
    // engine selection inside transcribe_detailed). Restored afterwards.
    let needs_override = effective_model != base_settings.selected_model
        || effective_language != base_settings.selected_language;
    if needs_override {
        let mut s = base_settings.clone();
        s.selected_model = effective_model.clone();
        s.selected_language = effective_language.clone();
        write_settings(app_handle, s);
    }

    let result = run_engine(
        app_handle,
        input_str,
        &effective_model,
        diarize,
        speaker_hint,
    );

    if needs_override {
        write_settings(app_handle, base_settings);
    }

    result
}

fn run_engine(
    app_handle: &AppHandle,
    input_str: &str,
    model_id: &str,
    diarize: bool,
    speaker_hint: Option<usize>,
) -> Result<TranscriptionDetails> {
    use crate::audio_toolkit::audio::read_wav_samples;

    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();
    let temp_wav = std::env::temp_dir().join(format!("echo_cli_{}.wav", ts));

    println!("[*] Extracting audio via ffmpeg (16kHz mono)...");
    let status = Command::new("ffmpeg")
        .args([
            "-i",
            input_str,
            "-ar",
            &TARGET_SAMPLE_RATE.to_string(),
            "-ac",
            "1",
            "-f",
            "wav",
            "-vn",
            temp_wav.to_str().context("Temp path is not valid UTF-8")?,
            "-y",
        ])
        .status()
        .context("Failed to execute ffmpeg")?;
    if !status.success() {
        anyhow::bail!("ffmpeg failed to extract audio");
    }

    println!("[*] Loading audio samples...");
    let samples = read_wav_samples(&temp_wav).context("Failed to read WAV samples")?;
    let _ = std::fs::remove_file(&temp_wav);

    let manager = app_handle.state::<Arc<TranscriptionManager>>();
    if !manager.is_model_loaded() {
        println!("[*] Loading model: {}...", model_id);
        manager
            .load_model(model_id)
            .context("Failed to load transcription model")?;
    }

    println!("[*] Transcribing (this may take a while for large files)...");
    let mut details = manager
        .transcribe_detailed(samples.clone())
        .context("Transcription failed")?;

    if diarize {
        // Phase 2 attaches real turns here. Until diarization::diarize is
        // implemented it returns an error or empty; degrade gracefully — keep
        // timestamps, drop speakers, warn on stderr.
        match crate::diarization::diarize(&samples, TARGET_SAMPLE_RATE, speaker_hint) {
            Ok(turns) if !turns.is_empty() => details.speakers = Some(turns),
            Ok(_) => {
                eprintln!("[!] Diarization produced no speaker turns; output has timestamps only.")
            }
            Err(e) => eprintln!("[!] Diarization failed ({e}); output has timestamps only."),
        }
    }

    Ok(details)
}
