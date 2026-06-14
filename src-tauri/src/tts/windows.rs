use windows::core::HSTRING;
use windows::Media::SpeechSynthesis::SpeechSynthesizer;
use windows::Storage::Streams::DataReader;

use super::{TtsEngine, VoiceInfo};

/// TTS via the built-in WinRT synthesizer. A fresh `SpeechSynthesizer` is
/// created per call: construction is cheap and it sidesteps apartment/threading
/// questions for an object held across tauri command threads.
pub struct WindowsTts;

impl WindowsTts {
    pub fn new() -> Self {
        Self
    }
}

impl TtsEngine for WindowsTts {
    fn list_voices(&self) -> Result<Vec<VoiceInfo>, String> {
        let voices = SpeechSynthesizer::AllVoices().map_err(|e| e.to_string())?;
        let mut out = Vec::new();
        for v in &voices {
            out.push(VoiceInfo {
                id: v.Id().map_err(|e| e.to_string())?.to_string(),
                display_name: v.DisplayName().map_err(|e| e.to_string())?.to_string(),
                language: v.Language().map_err(|e| e.to_string())?.to_string(),
            });
        }
        Ok(out)
    }

    fn synthesize(&self, text: &str, voice_id: Option<&str>) -> Result<Vec<u8>, String> {
        let synth = SpeechSynthesizer::new().map_err(|e| e.to_string())?;

        if let Some(id) = voice_id {
            let voices = SpeechSynthesizer::AllVoices().map_err(|e| e.to_string())?;
            let target = voices
                .into_iter()
                .find(|v| v.Id().map(|h| h.to_string() == id).unwrap_or(false))
                .ok_or_else(|| format!("TTS voice not found: {id}"))?;
            synth.SetVoice(&target).map_err(|e| e.to_string())?;
        }

        // The synthesizer emits a complete WAV container, which rodio decodes.
        let stream = synth
            .SynthesizeTextToStreamAsync(&HSTRING::from(text))
            .map_err(|e| e.to_string())?
            .get()
            .map_err(|e| e.to_string())?;

        let size = stream.Size().map_err(|e| e.to_string())?;
        let size: u32 = size
            .try_into()
            .map_err(|_| "synthesized audio exceeds 4 GiB".to_string())?;

        let input = stream.GetInputStreamAt(0).map_err(|e| e.to_string())?;
        let reader = DataReader::CreateDataReader(&input).map_err(|e| e.to_string())?;
        reader
            .LoadAsync(size)
            .map_err(|e| e.to_string())?
            .get()
            .map_err(|e| e.to_string())?;

        let mut wav = vec![0u8; size as usize];
        reader.ReadBytes(&mut wav).map_err(|e| e.to_string())?;
        Ok(wav)
    }
}
