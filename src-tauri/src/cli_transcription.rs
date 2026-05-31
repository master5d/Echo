use crate::audio_toolkit::audio::read_wav_samples;
use crate::managers::model::{EngineType, ModelManager};
use crate::managers::transcription::TranscriptionManager;
use crate::settings::{get_settings, write_settings};
use anyhow::{Context, Result};
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::Arc;
use tauri::{AppHandle, Manager};

/// Headless offline transcription of an audio/video file.
///
/// Reuses the running engine's `TranscriptionManager::transcribe`, which already
/// applies the bilingual RU/EN code-switch steering (mixed-script initial prompt,
/// glossary, tuned `no_speech_thold`) for Whisper models when the language is
/// "auto" or Slavic — so the CLI output matches the GUI's quality.
pub fn run_cli_transcription(
    app_handle: &AppHandle,
    input: &Path,
    output: Option<&Path>,
    language: Option<&str>,
    model: Option<&str>,
) -> Result<()> {
    println!("[*] Starting CLI transcription...");
    println!("[*] Input: {}", input.display());

    let input_str = input.to_str().context("Input path is not valid UTF-8")?;

    // Verify ffmpeg up front — a clear message beats a cryptic spawn failure.
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

    // Resolve the effective model and warn if it can't code-switch RU/EN.
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

    // Temporarily steer settings for this run (language + model drive the
    // bilingual prompt and engine selection inside transcribe). Restored after.
    let needs_override = effective_model != base_settings.selected_model
        || effective_language != base_settings.selected_language;
    if needs_override {
        let mut s = base_settings.clone();
        s.selected_model = effective_model.clone();
        s.selected_language = effective_language.clone();
        write_settings(app_handle, s);
    }

    // Run the pipeline, capturing the result so we can always restore settings.
    let result = transcribe_file(app_handle, input_str, &effective_model);

    if needs_override {
        write_settings(app_handle, base_settings);
    }

    let text = result?;

    if let Some(out_path) = output {
        std::fs::write(out_path, &text).context("Failed to write output file")?;
        println!("[+] Transcription saved to: {}", out_path.display());
    } else {
        println!("--- TRANSCRIPTION START ---");
        println!("{}", text);
        println!("--- TRANSCRIPTION END ---");
    }

    Ok(())
}

/// Extract → load model → transcribe. Split out so the caller can restore
/// settings regardless of success/failure.
fn transcribe_file(app_handle: &AppHandle, input_str: &str, model_id: &str) -> Result<String> {
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
            "16000",
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
    manager.transcribe(samples).context("Transcription failed")
}
