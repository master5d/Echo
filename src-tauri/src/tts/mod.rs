use rodio::{OutputStreamBuilder, Sink};
use serde::{Deserialize, Serialize};
use specta::Type;
use std::io::Cursor;
use std::sync::{Arc, Mutex};

#[cfg(windows)]
mod windows;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct VoiceInfo {
    pub id: String,
    pub display_name: String,
    pub language: String,
}

pub trait TtsEngine: Send + Sync {
    fn list_voices(&self) -> Result<Vec<VoiceInfo>, String>;
    fn synthesize(&self, text: &str, voice_id: Option<&str>, rate: f32) -> Result<Vec<u8>, String>;
}

/// Pick a voice whose language matches the text's script. If >=30% of the
/// alphabetic characters are Cyrillic, prefer a `ru*` voice; otherwise an
/// `en*` voice. Falls back to the first available voice.
pub fn pick_voice_for_text<'a>(text: &str, voices: &'a [VoiceInfo]) -> Option<&'a VoiceInfo> {
    let alpha: Vec<char> = text.chars().filter(|c| c.is_alphabetic()).collect();
    if alpha.is_empty() {
        return voices.first();
    }

    let cyrillic = alpha
        .iter()
        .filter(|&&c| ('\u{0400}'..='\u{04FF}').contains(&c))
        .count();
    let is_russian = (cyrillic as f32 / alpha.len() as f32) >= 0.3;

    let prefix = if is_russian { "ru" } else { "en" };
    voices
        .iter()
        .find(|v| v.language.to_lowercase().starts_with(prefix))
        .or_else(|| voices.first())
}

pub struct TtsManager {
    engine: Option<Box<dyn TtsEngine>>,
    current_sink: Arc<Mutex<Option<Arc<Sink>>>>,
}

impl TtsManager {
    pub fn new() -> Self {
        #[cfg(windows)]
        {
            Self {
                engine: Some(Box::new(windows::WindowsTts::new())),
                current_sink: Arc::new(Mutex::new(None)),
            }
        }
        #[cfg(not(windows))]
        {
            Self {
                engine: None,
                current_sink: Arc::new(Mutex::new(None)),
            }
        }
    }

    pub fn list_voices(&self) -> Result<Vec<VoiceInfo>, String> {
        self.engine
            .as_ref()
            .ok_or_else(|| "no TTS engine available on this platform".to_string())?
            .list_voices()
    }

    pub fn speak(&self, text: String, voice_id: Option<String>, rate: f32) -> Result<(), String> {
        let engine = self
            .engine
            .as_ref()
            .ok_or_else(|| "no TTS engine available on this platform".to_string())?;

        // Stop current playback
        self.stop()?;

        let wav_bytes = engine.synthesize(&text, voice_id.as_deref(), rate)?;

        let current_sink = self.current_sink.clone();

        // Playback in a separate thread to not block
        std::thread::spawn(move || {
            if let Err(e) = Self::play_wav(wav_bytes, current_sink) {
                log::error!("Failed to play TTS audio: {}", e);
            }
        });

        Ok(())
    }

    pub fn stop(&self) -> Result<(), String> {
        let mut sink_lock = self.current_sink.lock().map_err(|e| e.to_string())?;
        if let Some(sink) = sink_lock.take() {
            sink.stop();
        }
        Ok(())
    }

    pub(crate) fn play_wav(
        wav_bytes: Vec<u8>,
        current_sink: Arc<Mutex<Option<Arc<Sink>>>>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let cursor = Cursor::new(wav_bytes);

        let stream_builder = OutputStreamBuilder::from_default_device()?;
        let stream_handle = stream_builder.open_stream()?;
        let mixer = stream_handle.mixer();

        // rodio::play in this fork handles decoding if passed a Read + Seek
        let sink = Arc::new(rodio::play(mixer, cursor)?);

        {
            let mut sink_lock = current_sink.lock().map_err(|e| e.to_string())?;
            *sink_lock = Some(sink.clone());
        }

        sink.sleep_until_end();

        Ok(())
    }
}

#[cfg(test)]
mod voice_pick_tests {
    use super::*;

    fn voices() -> Vec<VoiceInfo> {
        vec![
            VoiceInfo {
                id: "en-1".into(),
                display_name: "David".into(),
                language: "en-US".into(),
            },
            VoiceInfo {
                id: "ru-1".into(),
                display_name: "Irina".into(),
                language: "ru-RU".into(),
            },
        ]
    }

    #[test]
    fn russian_text_picks_ru_voice() {
        let v = voices();
        let picked = pick_voice_for_text("Привет, как дела?", &v).unwrap();
        assert_eq!(picked.id, "ru-1");
    }

    #[test]
    fn english_text_picks_en_voice() {
        let v = voices();
        let picked = pick_voice_for_text("Hello, how are you?", &v).unwrap();
        assert_eq!(picked.id, "en-1");
    }

    #[test]
    fn no_alpha_falls_back_to_first() {
        let v = voices();
        let picked = pick_voice_for_text("12345 !!!", &v).unwrap();
        assert_eq!(picked.id, "en-1");
    }
}

#[cfg(all(test, windows))]
mod tests {
    use super::*;

    /// Real WinRT synthesis — needs installed voices, so opt-in only:
    /// cargo test --lib tts -- --ignored --nocapture
    #[test]
    #[ignore]
    fn windows_tts_synthesizes_wav() {
        let engine = windows::WindowsTts::new();

        let voices = engine.list_voices().expect("list_voices failed");
        assert!(!voices.is_empty(), "no Windows voices installed");
        println!(
            "voices: {:?}",
            voices.iter().map(|v| &v.display_name).collect::<Vec<_>>()
        );

        let wav = engine
            .synthesize("Echo speech engine online. Эхо на связи.", None, 1.0)
            .expect("synthesize failed");
        assert!(wav.len() > 44, "WAV too small: {} bytes", wav.len());
        assert_eq!(&wav[0..4], b"RIFF", "not a WAV container");
        println!("synthesized {} bytes", wav.len());
    }
}
