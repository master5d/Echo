use crate::file_transcription::transcribe_file_detailed;
use crate::transcript_format::OutputFormat;
use std::path::PathBuf;
use tauri::AppHandle;

/// Transcribe a file from the GUI and return the rendered string.
/// `format` is one of: plain|inline|srt|vtt|json.
#[tauri::command]
#[specta::specta]
pub async fn transcribe_file_to_string(
    app_handle: AppHandle,
    path: String,
    language: Option<String>,
    model: Option<String>,
    diarize: bool,
    speaker_hint: Option<u32>,
    format: String,
) -> Result<String, String> {
    let fmt =
        OutputFormat::from_cli(&format).ok_or_else(|| format!("Unknown format '{format}'"))?;
    let input = PathBuf::from(path);
    let want_words = diarize || fmt.is_word_level();
    // Run the blocking pipeline off the async runtime thread.
    let details = tauri::async_runtime::spawn_blocking(move || {
        transcribe_file_detailed(
            &app_handle,
            &input,
            language.as_deref(),
            model.as_deref(),
            diarize,
            speaker_hint.map(|n| n as usize),
            want_words,
        )
    })
    .await
    .map_err(|e| format!("task join error: {e}"))?
    .map_err(|e| e.to_string())?;

    let body = match fmt {
        OutputFormat::Json if details.words.is_some() => {
            crate::transcript_format::render_word_json(
                details.words.as_deref().unwrap(),
                details.speakers.as_deref(),
            )
        }
        OutputFormat::Karaoke => crate::transcript_format::render_karaoke(
            details.words.as_deref().unwrap_or(&[]),
            details.speakers.as_deref(),
        ),
        _ => crate::transcript_format::render(
            &details.text,
            &details.segments,
            details.words.as_deref(),
            details.speakers.as_deref(),
            fmt,
        ),
    };

    Ok(body)
}

/// Request cancellation of the in-flight file transcription (Transcribe-file tab).
#[tauri::command]
#[specta::specta]
pub async fn cancel_file_transcription(app: AppHandle) -> Result<(), String> {
    use crate::managers::transcription::TranscriptionManager;
    use std::sync::Arc;
    use tauri::Manager;
    app.state::<Arc<TranscriptionManager>>().request_cancel();
    Ok(())
}
