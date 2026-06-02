//! The `transcription-progress` Tauri event payload for the Transcribe-file UI.

use serde::Serialize;
use tauri::{AppHandle, Emitter};

/// Pipeline phase reported to the UI. Serializes to a lowercase snake_case string.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProgressPhase {
    Decoding,
    LoadingModel,
    Transcribing,
    Diarizing,
    Formatting,
    Done,
}

/// Payload of the `transcription-progress` event. `percent` is None for
/// indeterminate phases.
#[derive(Debug, Clone, Serialize)]
pub struct TranscriptionProgress {
    pub phase: ProgressPhase,
    pub percent: Option<u8>,
}

/// Emit a `transcription-progress` event. Best-effort (ignores emit errors).
pub fn emit_progress(app: &AppHandle, phase: ProgressPhase, percent: Option<u8>) {
    let _ = app.emit(
        "transcription-progress",
        TranscriptionProgress { phase, percent },
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn phase_serializes_snake_case() {
        assert_eq!(
            serde_json::to_string(&ProgressPhase::LoadingModel).unwrap(),
            "\"loading_model\""
        );
        assert_eq!(
            serde_json::to_string(&ProgressPhase::Transcribing).unwrap(),
            "\"transcribing\""
        );
    }

    #[test]
    fn payload_shape() {
        let p = TranscriptionProgress {
            phase: ProgressPhase::Transcribing,
            percent: Some(42),
        };
        let j = serde_json::to_string(&p).unwrap();
        assert!(j.contains("\"phase\":\"transcribing\""));
        assert!(j.contains("\"percent\":42"));
        let none = TranscriptionProgress {
            phase: ProgressPhase::Diarizing,
            percent: None,
        };
        assert!(serde_json::to_string(&none)
            .unwrap()
            .contains("\"percent\":null"));
    }
}
