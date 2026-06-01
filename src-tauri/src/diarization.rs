//! Speaker diarization façade. Phase 1 ships this stub (returns an error so the
//! file pipeline degrades gracefully to timestamps-only). Phase 2 replaces the
//! body with a burn-based pyannote pipeline. The signature is stable.

use crate::transcript_format::SpeakerTurn;
use anyhow::Result;

/// Diarize 16kHz mono samples into speaker turns. `hint` is an optional known
/// speaker count. Phase 1 stub: not yet implemented.
pub fn diarize(
    _samples: &[f32],
    _sample_rate: u32,
    _hint: Option<usize>,
) -> Result<Vec<SpeakerTurn>> {
    anyhow::bail!("speaker diarization is not yet available in this build")
}
