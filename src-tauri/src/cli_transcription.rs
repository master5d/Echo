//! CLI offline transcription: thin wrapper over the shared file-transcription
//! core that renders the result in the requested format to stdout or a file.

use crate::file_transcription::transcribe_file_detailed;
use crate::transcript_format::{render, OutputFormat};
use anyhow::{Context, Result};
use std::path::Path;
use tauri::AppHandle;

#[allow(clippy::too_many_arguments)]
pub fn run_cli_transcription(
    app_handle: &AppHandle,
    input: &Path,
    output: Option<&Path>,
    language: Option<&str>,
    model: Option<&str>,
    format: Option<&str>,
    diarize: bool,
    speaker_hint: Option<usize>,
) -> Result<()> {
    println!("[*] Starting CLI transcription...");
    println!("[*] Input: {}", input.display());

    if speaker_hint.is_some() {
        if !diarize {
            eprintln!("[!] --speakers was given without --diarize; ignoring the speaker count.");
        } else {
            eprintln!(
                "[!] --speakers is not yet supported by the diarization engine (speaker count \
is auto-detected); the value is ignored."
            );
        }
    }

    // Resolve output format: explicit --format wins, else infer from -o extension, else plain.
    let fmt = match format {
        Some(f) => OutputFormat::from_cli(f)
            .with_context(|| format!("Unknown --format '{f}'. Use plain|inline|srt|vtt|json."))?,
        None => output
            .and_then(|p| p.extension())
            .and_then(|e| e.to_str())
            .and_then(OutputFormat::from_extension)
            .unwrap_or(OutputFormat::Plain),
    };

    let details =
        transcribe_file_detailed(app_handle, input, language, model, diarize, speaker_hint)?;
    let rendered = render(
        &details.text,
        &details.segments,
        details.words.as_deref(),
        details.speakers.as_deref(),
        fmt,
    );

    if let Some(out_path) = output {
        std::fs::write(out_path, &rendered).context("Failed to write output file")?;
        println!("[+] Transcription saved to: {}", out_path.display());
    } else {
        println!("--- TRANSCRIPTION START ---");
        println!("{}", rendered);
        println!("--- TRANSCRIPTION END ---");
    }

    Ok(())
}
