use std::io::Cursor;
use std::sync::{Arc, Mutex};
use serde::{Deserialize, Serialize};
use specta::Type;
use rodio::{OutputStreamBuilder, Sink};

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
    fn synthesize(&self, text: &str, voice_id: Option<&str>) -> Result<Vec<u8>, String>;
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

    pub fn speak(&self, text: String, voice_id: Option<String>) -> Result<(), String> {
        let engine = self.engine
            .as_ref()
            .ok_or_else(|| "no TTS engine available on this platform".to_string())?;

        // Stop current playback
        self.stop()?;

        let wav_bytes = engine.synthesize(&text, voice_id.as_deref())?;

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

    pub(crate) fn play_wav(wav_bytes: Vec<u8>, current_sink: Arc<Mutex<Option<Arc<Sink>>>>) -> Result<(), Box<dyn std::error::Error>> {
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
            .synthesize("Echo speech engine online. Эхо на связи.", None)
            .expect("synthesize failed");
        assert!(wav.len() > 44, "WAV too small: {} bytes", wav.len());
        assert_eq!(&wav[0..4], b"RIFF", "not a WAV container");
        println!("synthesized {} bytes", wav.len());
    }
}
