//! Pure transcript formatting: turns engine segments (+ optional speaker turns)
//! into plain text, inline timestamps, SRT, WebVTT, or JSON. No I/O, no engines.

use serde::Serialize;

/// One transcript segment with start/end in seconds (from the ASR engine).
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TimedSegment {
    pub start: f32,
    pub end: f32,
    pub text: String,
}

/// Zero-based speaker index. Rendered to humans as `Speaker {id+1}`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct SpeakerId(pub u32);

/// A contiguous interval attributed to one speaker (from diarization).
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SpeakerTurn {
    pub start: f32,
    pub end: f32,
    pub speaker: SpeakerId,
}

/// Output format selected via CLI `--format` or GUI dropdown.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    Plain,
    Inline,
    Srt,
    Vtt,
    Json,
}

impl OutputFormat {
    /// Parse the CLI `--format` value. Case-insensitive. Returns None if unknown.
    pub fn from_cli(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "plain" | "txt" | "text" => Some(Self::Plain),
            "inline" => Some(Self::Inline),
            "srt" => Some(Self::Srt),
            "vtt" | "webvtt" => Some(Self::Vtt),
            "json" => Some(Self::Json),
            _ => None,
        }
    }

    /// Infer a format from an output file extension (used when `--format` is
    /// omitted but `-o foo.srt` is given). Unknown extensions → None.
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_ascii_lowercase().as_str() {
            "srt" => Some(Self::Srt),
            "vtt" => Some(Self::Vtt),
            "json" => Some(Self::Json),
            "txt" | "text" => Some(Self::Plain),
            _ => None,
        }
    }
}

/// Human label for a speaker, 1-based: SpeakerId(0) -> "Speaker 1".
pub fn speaker_label(id: SpeakerId) -> String {
    format!("Speaker {}", id.0 + 1)
}

/// `HH:MM:SS,mmm` (SRT uses a comma before milliseconds).
pub fn fmt_srt(secs: f32) -> String {
    let (h, m, s, ms) = hmsms(secs);
    format!("{:02}:{:02}:{:02},{:03}", h, m, s, ms)
}

/// `HH:MM:SS.mmm` (WebVTT uses a dot before milliseconds).
pub fn fmt_vtt(secs: f32) -> String {
    let (h, m, s, ms) = hmsms(secs);
    format!("{:02}:{:02}:{:02}.{:03}", h, m, s, ms)
}

/// `MM:SS` for inline prefixes. Minutes may exceed 59 (e.g. `75:30`) so a single
/// glance reads as "minutes:seconds" without hour parsing.
pub fn fmt_inline(secs: f32) -> String {
    let total = secs.max(0.0).round() as u64;
    format!("{:02}:{:02}", total / 60, total % 60)
}

fn hmsms(secs: f32) -> (u64, u64, u64, u64) {
    let clamped = secs.max(0.0);
    let total_ms = (clamped * 1000.0).round() as u64;
    let ms = total_ms % 1000;
    let total_s = total_ms / 1000;
    (total_s / 3600, (total_s % 3600) / 60, total_s % 60, ms)
}

/// Assign a speaker to each segment by maximum time overlap with the turns.
/// Segments with no overlapping turn get `None`. Output is index-aligned to
/// `segments`. Pure and deterministic.
pub fn assign_speakers(segments: &[TimedSegment], turns: &[SpeakerTurn]) -> Vec<Option<SpeakerId>> {
    segments
        .iter()
        .map(|seg| {
            let mut best: Option<(f32, SpeakerId)> = None;
            for turn in turns {
                let overlap = (seg.end.min(turn.end) - seg.start.max(turn.start)).max(0.0);
                if overlap > 0.0 && best.map(|(o, _)| overlap > o).unwrap_or(true) {
                    best = Some((overlap, turn.speaker));
                }
            }
            best.map(|(_, sp)| sp)
        })
        .collect()
}

/// JSON shape for `OutputFormat::Json`.
#[derive(Serialize)]
struct JsonSegment {
    start: f32,
    end: f32,
    text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    speaker: Option<String>,
}

#[derive(Serialize)]
struct JsonDoc {
    segments: Vec<JsonSegment>,
}

/// Render segments (+ optional speaker turns) into the chosen format.
///
/// `plain_text` is the already-joined, post-processed transcript used for
/// `Plain` (preserves current CLI behavior exactly). Other formats build from
/// `segments`.
pub fn render(
    plain_text: &str,
    segments: &[TimedSegment],
    speakers: Option<&[SpeakerTurn]>,
    format: OutputFormat,
) -> String {
    let labels: Vec<Option<SpeakerId>> = match speakers {
        Some(turns) => assign_speakers(segments, turns),
        None => vec![None; segments.len()],
    };
    let prefix = |i: usize| -> String {
        labels
            .get(i)
            .copied()
            .flatten()
            .map(|sp| format!("{}: ", speaker_label(sp)))
            .unwrap_or_default()
    };

    match format {
        OutputFormat::Plain => plain_text.to_string(),

        OutputFormat::Inline => segments
            .iter()
            .enumerate()
            .map(|(i, s)| format!("[{}] {}{}", fmt_inline(s.start), prefix(i), s.text.trim()))
            .collect::<Vec<_>>()
            .join("\n"),

        OutputFormat::Srt => segments
            .iter()
            .enumerate()
            .map(|(i, s)| {
                format!(
                    "{}\n{} --> {}\n{}{}\n",
                    i + 1,
                    fmt_srt(s.start),
                    fmt_srt(s.end),
                    prefix(i),
                    s.text.trim()
                )
            })
            .collect::<Vec<_>>()
            .join("\n"),

        OutputFormat::Vtt => {
            let mut out = String::from("WEBVTT\n\n");
            for (i, s) in segments.iter().enumerate() {
                let body = match labels.get(i).copied().flatten() {
                    Some(sp) => format!("<v {}>{}", speaker_label(sp), s.text.trim()),
                    None => s.text.trim().to_string(),
                };
                out.push_str(&format!(
                    "{} --> {}\n{}\n\n",
                    fmt_vtt(s.start),
                    fmt_vtt(s.end),
                    body
                ));
            }
            out.trim_end().to_string()
        }

        OutputFormat::Json => {
            let docs = JsonDoc {
                segments: segments
                    .iter()
                    .enumerate()
                    .map(|(i, s)| JsonSegment {
                        start: s.start,
                        end: s.end,
                        text: s.text.trim().to_string(),
                        speaker: labels.get(i).copied().flatten().map(speaker_label),
                    })
                    .collect(),
            };
            serde_json::to_string_pretty(&docs).unwrap_or_else(|_| "{\"segments\":[]}".to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn seg(start: f32, end: f32, text: &str) -> TimedSegment {
        TimedSegment {
            start,
            end,
            text: text.to_string(),
        }
    }

    #[test]
    fn timecodes_format_correctly() {
        assert_eq!(fmt_srt(0.0), "00:00:00,000");
        assert_eq!(fmt_srt(12.5), "00:00:12,500");
        assert_eq!(fmt_srt(3661.0), "01:01:01,000");
        assert_eq!(fmt_vtt(12.5), "00:00:12.500");
        assert_eq!(fmt_inline(75.4), "01:15");
        assert_eq!(fmt_inline(0.0), "00:00");
    }

    #[test]
    fn plain_returns_joined_text_verbatim() {
        let segs = vec![seg(0.0, 1.0, "hello"), seg(1.0, 2.0, "world")];
        assert_eq!(
            render("hello world", &segs, None, OutputFormat::Plain),
            "hello world"
        );
    }

    #[test]
    fn inline_prefixes_timestamps() {
        let segs = vec![seg(0.0, 2.0, "привет"), seg(12.0, 14.0, "world")];
        let out = render("привет world", &segs, None, OutputFormat::Inline);
        assert_eq!(out, "[00:00] привет\n[00:12] world");
    }

    #[test]
    fn srt_numbers_and_arrows() {
        let segs = vec![seg(0.0, 2.5, "hi")];
        let out = render("hi", &segs, None, OutputFormat::Srt);
        assert_eq!(out, "1\n00:00:00,000 --> 00:00:02,500\nhi\n");
    }

    #[test]
    fn vtt_has_header_and_dot_separator() {
        let segs = vec![seg(1.0, 2.0, "hi")];
        let out = render("hi", &segs, None, OutputFormat::Vtt);
        assert!(out.starts_with("WEBVTT\n\n"));
        assert!(out.contains("00:00:01.000 --> 00:00:02.000"));
    }

    #[test]
    fn json_includes_segments() {
        let segs = vec![seg(0.0, 1.0, "hi")];
        let out = render("hi", &segs, None, OutputFormat::Json);
        assert!(out.contains("\"segments\""));
        assert!(out.contains("\"text\": \"hi\""));
        assert!(!out.contains("\"speaker\"")); // omitted when no speakers
    }

    #[test]
    fn assign_speakers_by_max_overlap() {
        let segs = vec![seg(0.0, 2.0, "a"), seg(5.0, 7.0, "b"), seg(20.0, 21.0, "c")];
        let turns = vec![
            SpeakerTurn {
                start: 0.0,
                end: 3.0,
                speaker: SpeakerId(0),
            },
            SpeakerTurn {
                start: 4.0,
                end: 8.0,
                speaker: SpeakerId(1),
            },
        ];
        let got = assign_speakers(&segs, &turns);
        assert_eq!(got, vec![Some(SpeakerId(0)), Some(SpeakerId(1)), None]);
    }

    #[test]
    fn straddling_segment_takes_larger_overlap() {
        // segment 2.0..4.0 overlaps spk0 by 1.0 (2..3) and spk1 by 1.0... tie ->
        // make spk1 larger: spk1 covers 2.5..4.0 => overlap 1.5 > 1.0.
        let segs = vec![seg(2.0, 4.0, "x")];
        let turns = vec![
            SpeakerTurn {
                start: 0.0,
                end: 3.0,
                speaker: SpeakerId(0),
            },
            SpeakerTurn {
                start: 2.5,
                end: 6.0,
                speaker: SpeakerId(1),
            },
        ];
        assert_eq!(assign_speakers(&segs, &turns), vec![Some(SpeakerId(1))]);
    }

    #[test]
    fn speaker_labels_render_in_each_format() {
        let segs = vec![seg(0.0, 2.0, "hi"), seg(2.0, 4.0, "yo")];
        let turns = vec![
            SpeakerTurn {
                start: 0.0,
                end: 2.0,
                speaker: SpeakerId(0),
            },
            SpeakerTurn {
                start: 2.0,
                end: 4.0,
                speaker: SpeakerId(1),
            },
        ];
        let inline = render("hi yo", &segs, Some(&turns), OutputFormat::Inline);
        assert_eq!(inline, "[00:00] Speaker 1: hi\n[00:02] Speaker 2: yo");
        let vtt = render("hi yo", &segs, Some(&turns), OutputFormat::Vtt);
        assert!(vtt.contains("<v Speaker 1>hi"));
        let srt = render("hi yo", &segs, Some(&turns), OutputFormat::Srt);
        assert!(srt.contains("Speaker 2: yo"));
    }
}
