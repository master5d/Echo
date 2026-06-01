//! Formatting logic for transcription output (plain, inline, SRT, VTT, JSON)
//! with speaker diarization support. Pure logic, fully unit-tested.

use serde::{Deserialize, Serialize};
use std::fmt::Write;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TimedSegment {
    pub start: f32, // Seconds
    pub end: f32,
    pub text: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct SpeakerId(pub usize);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SpeakerTurn {
    pub start: f32,
    pub end: f32,
    pub speaker: SpeakerId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    Plain,
    Inline,
    Srt,
    Vtt,
    Json,
}

impl OutputFormat {
    pub fn from_cli(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "plain" => Some(Self::Plain),
            "inline" => Some(Self::Inline),
            "srt" => Some(Self::Srt),
            "vtt" => Some(Self::Vtt),
            "json" => Some(Self::Json),
            _ => None,
        }
    }

    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "txt" => Some(Self::Plain),
            "srt" => Some(Self::Srt),
            "vtt" => Some(Self::Vtt),
            "json" => Some(Self::Json),
            _ => None,
        }
    }
}

/// Helper: Format seconds as HH:MM:SS,mmm (SRT style) or HH:MM:SS.mmm (VTT)
fn format_timestamp(seconds: f32, srt: bool) -> String {
    let millis = (seconds.fract() * 1000.0) as u32;
    let total_secs = seconds.trunc() as u32;
    let hours = total_secs / 3600;
    let minutes = (total_secs % 3600) / 60;
    let secs = total_secs % 60;
    let sep = if srt { ',' } else { '.' };
    format!("{:02}:{:02}:{:02}{}{:03}", hours, minutes, secs, sep, millis)
}

/// Helper: Format seconds as [mm:ss] for inline display
fn format_short_timestamp(seconds: f32) -> String {
    let total_secs = seconds.trunc() as u32;
    let minutes = total_secs / 60;
    let secs = total_secs % 60;
    format!("[{:02}:{:02}]", minutes, secs)
}

/// Assigns speaker IDs to timed segments based on overlapping/nearest speaker turns.
pub fn assign_speakers(
    segments: &[TimedSegment],
    turns: &[SpeakerTurn],
) -> Vec<(TimedSegment, Option<SpeakerId>)> {
    segments
        .iter()
        .map(|seg| {
            // Find the speaker turn that overlaps most with this segment center
            let mid = (seg.start + seg.end) / 2.0;
            let speaker = turns
                .iter()
                .find(|turn| mid >= turn.start && mid <= turn.end)
                .or_else(|| {
                    // Fallback: find nearest turn if no direct overlap at midpoint
                    turns.iter().min_by(|a, b| {
                        let dist_a = (mid - (a.start + a.end) / 2.0).abs();
                        let dist_b = (mid - (b.start + b.end) / 2.0).abs();
                        dist_a.partial_cmp(&dist_b).unwrap()
                    })
                })
                .map(|t| t.speaker);
            (seg.clone(), speaker)
        })
        .collect()
}

pub fn render(
    plain_text: &str,
    segments: &[TimedSegment],
    turns: Option<&[SpeakerTurn]>,
    format: OutputFormat,
) -> String {
    match format {
        OutputFormat::Plain => plain_text.to_string(),
        OutputFormat::Json => serde_json::to_string_pretty(&segments).unwrap_or_default(),
        OutputFormat::Inline => {
            let mut out = String::new();
            let mut last_speaker = None;
            let speaker_segments = turns.map(|t| assign_speakers(segments, t));

            if let Some(ss) = speaker_segments {
                for (seg, speaker) in ss {
                    if speaker != last_speaker {
                        if let Some(id) = speaker {
                            write!(out, "\nSpeaker {}: ", id.0 + 1).ok();
                        }
                        last_speaker = speaker;
                    }
                    write!(out, "{} {} ", format_short_timestamp(seg.start), seg.text.trim()).ok();
                }
            } else {
                for seg in segments {
                    write!(out, "{} {} ", format_short_timestamp(seg.start), seg.text.trim()).ok();
                }
            }
            out.trim().to_string()
        }
        OutputFormat::Srt | OutputFormat::Vtt => {
            let mut out = String::new();
            if format == OutputFormat::Vtt {
                out.push_str("WEBVTT\n\n");
            }
            let is_srt = format == OutputFormat::Srt;
            let speaker_segments = turns.map(|t| assign_speakers(segments, t));

            for (i, seg) in segments.iter().enumerate() {
                if is_srt {
                    write!(out, "{}\n", i + 1).ok();
                }
                let speaker_label = speaker_segments.as_ref().and_then(|ss| {
                    ss[i].1.map(|id| format!("Speaker {}: ", id.0 + 1))
                }).unwrap_or_default();

                write!(
                    out,
                    "{} --> {}\n{}{}\n\n",
                    format_timestamp(seg.start, is_srt),
                    format_timestamp(seg.end, is_srt),
                    speaker_label,
                    seg.text.trim()
                )
                .ok();
            }
            out.trim().to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timestamp_formatting() {
        assert_eq!(format_timestamp(61.5, true), "00:01:01,500");
        assert_eq!(format_timestamp(61.5, false), "00:01:01.500");
        assert_eq!(format_short_timestamp(61.5), "[01:01]");
    }

    #[test]
    fn test_render_plain() {
        let segs = vec![TimedSegment { start: 0.0, end: 1.0, text: "Hello".into() }];
        assert_eq!(render("Hello", &segs, None, OutputFormat::Plain), "Hello");
    }

    #[test]
    fn test_render_srt_with_speakers() {
        let segs = vec![
            TimedSegment { start: 0.0, end: 2.0, text: "Hi I am Alice".into() },
            TimedSegment { start: 2.1, end: 4.0, text: "And I am Bob".into() },
        ];
        let turns = vec![
            SpeakerTurn { start: 0.0, end: 2.0, speaker: SpeakerId(0) },
            SpeakerTurn { start: 2.0, end: 4.0, speaker: SpeakerId(1) },
        ];
        let output = render("", &segs, Some(&turns), OutputFormat::Srt);
        assert!(output.contains("1\n00:00:00,000 --> 00:00:02,000\nSpeaker 1: Hi I am Alice"));
        assert!(output.contains("2\n00:00:02,100 --> 00:00:04,000\nSpeaker 2: And I am Bob"));
    }
}
