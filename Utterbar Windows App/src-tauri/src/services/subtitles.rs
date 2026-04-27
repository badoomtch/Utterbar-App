//! Subtitle generation for file transcription.
//!
//! Produces `.srt` and `.vtt` strings from a transcript + total duration.
//!
//! ## Honest timing (read this before "improving" the precision)
//!
//! sherpa-rs 0.6.x's safe Rust wrapper exposes `recognizer.transcribe()
//! -> String` only — it discards the per-token timestamps the underlying
//! sherpa-onnx C API actually populates (`SherpaOnnxOfflineRecognizerResult`
//! has `timestamps: *mut f32` + `tokens_arr` but the Rust shim ignores
//! them). Reaching past the wrapper into the FFI to extract them is
//! invasive — would need a forked sherpa-rs or unsafe transmutes — and
//! is over-budget for the v0.11.0 beta.
//!
//! So this module produces **sentence-level subtitles distributed
//! proportionally across the audio's known total duration**. Each
//! subtitle cue spans roughly 5–8 seconds of speech and is timed by the
//! character-fraction of the transcript it represents. That gives:
//!
//! - usable cues for previewing or rough editing
//! - consistent file format (.srt / .vtt) that downstream tools
//!   (Premiere, Resolve, Subtitle Edit, ffmpeg) accept
//! - **NOT word-perfect timing**. A 60-second podcast where the speaker
//!   pauses for 10s mid-sentence will land a cue 5s out. The UI copy
//!   labels these as "approximate timing" so users aren't surprised.
//!
//! When `AsrResult.words` becomes populated (future provider work
//! that *does* round-trip the FFI timestamps), `from_words()` below is
//! the upgrade path: it groups words into cues at natural pause points
//! and uses real start/end timestamps. That function is wired but
//! currently unreached — the Rust wrappers all return `words: None`.

use crate::services::inference::WordTiming;

/// One subtitle cue. Times are in milliseconds from the start of the
/// audio.
#[derive(Debug, Clone)]
pub struct SubtitleCue {
    pub start_ms: u32,
    pub end_ms: u32,
    pub text: String,
}

/// Target cue length when distributing a transcript across the
/// timeline. 6 s is a common subtitle-editor default — long enough to
/// avoid flicker, short enough to keep the cue from outlasting the
/// shot in a typical edit.
const TARGET_CUE_MS: u32 = 6000;
const MIN_CUE_MS: u32 = 1500;
const MAX_CUE_MS: u32 = 9000;

/// Build cues from a plain transcript + the total audio duration. Used
/// when the ASR provider doesn't expose word-level timestamps (which
/// is everyone today; see module docs).
///
/// Splits on sentence boundaries first (`.!?`), then if any sentence
/// is much longer than `TARGET_CUE_MS` worth of characters, splits it
/// further on commas / clause boundaries. Each cue's duration is
/// proportional to its share of the total character count, clamped
/// to [MIN, MAX] so a single very-short sentence doesn't get a 0.2s
/// flash and a very-long one doesn't sit on screen for half a minute.
pub fn cues_from_text(text: &str, total_duration_ms: u32) -> Vec<SubtitleCue> {
    let trimmed = text.trim();
    if trimmed.is_empty() || total_duration_ms == 0 {
        return Vec::new();
    }

    // Estimate average chars-per-cue from the target cue length and
    // the total characters / total duration. We aim for ~TARGET_CUE_MS
    // per cue, scaled by how dense the transcript is.
    let total_chars = trimmed.chars().count().max(1) as f32;
    let chars_per_ms = total_chars / total_duration_ms as f32;
    let target_chars_per_cue = (chars_per_ms * TARGET_CUE_MS as f32).clamp(40.0, 400.0) as usize;

    let chunks = split_for_cues(trimmed, target_chars_per_cue);
    if chunks.is_empty() {
        return Vec::new();
    }

    // Distribute total duration across chunks by character count so a
    // long sentence gets more of the timeline than a short one.
    let chunk_chars: Vec<usize> = chunks.iter().map(|c| c.chars().count().max(1)).collect();
    let total_chunk_chars: usize = chunk_chars.iter().sum();

    let mut cues = Vec::with_capacity(chunks.len());
    let mut cursor_ms: u32 = 0;
    for (i, chunk) in chunks.iter().enumerate() {
        let share = chunk_chars[i] as f32 / total_chunk_chars as f32;
        let raw = (share * total_duration_ms as f32) as u32;
        let dur = raw.clamp(MIN_CUE_MS, MAX_CUE_MS);
        let start = cursor_ms;
        let end = (start + dur).min(total_duration_ms);
        cues.push(SubtitleCue {
            start_ms: start,
            end_ms: end,
            text: chunk.trim().to_string(),
        });
        cursor_ms = end;
        // Don't let cumulative drift push us past total — clamp on
        // the next iteration if needed.
        if cursor_ms >= total_duration_ms {
            break;
        }
    }

    // Snap the final cue's end to the total duration so the timeline
    // ends exactly where the audio does.
    if let Some(last) = cues.last_mut() {
        last.end_ms = total_duration_ms;
    }

    cues
}

/// Build cues from real word timings. Currently unreached because no
/// AsrProvider populates `words: Some(...)` — but this is the
/// honest-timing upgrade path the moment one does. Groups consecutive
/// words into cues at natural pause boundaries (>500ms gap) or
/// whenever the cue would exceed `TARGET_CUE_MS`.
#[allow(dead_code)]
pub fn cues_from_words(words: &[WordTiming]) -> Vec<SubtitleCue> {
    if words.is_empty() {
        return Vec::new();
    }
    const PAUSE_BREAK_MS: u32 = 500;

    let mut cues = Vec::new();
    let mut cur: Vec<&WordTiming> = Vec::new();
    let flush = |cur: &mut Vec<&WordTiming>, cues: &mut Vec<SubtitleCue>| {
        if cur.is_empty() {
            return;
        }
        let start = cur.first().unwrap().start_ms;
        let end = cur.last().unwrap().end_ms;
        let text = cur
            .iter()
            .map(|w| w.text.as_str())
            .collect::<Vec<_>>()
            .join(" ");
        cues.push(SubtitleCue {
            start_ms: start,
            end_ms: end,
            text,
        });
        cur.clear();
    };

    for w in words {
        if let Some(last) = cur.last() {
            let gap = w.start_ms.saturating_sub(last.end_ms);
            let cue_start = cur.first().unwrap().start_ms;
            let cue_dur = w.end_ms.saturating_sub(cue_start);
            if gap > PAUSE_BREAK_MS || cue_dur > TARGET_CUE_MS {
                flush(&mut cur, &mut cues);
            }
        }
        cur.push(w);
    }
    flush(&mut cur, &mut cues);
    cues
}

/// Split a transcript into chunks of roughly `target_chars` length,
/// preferring sentence boundaries and falling back to clause / comma
/// boundaries for very long sentences.
fn split_for_cues(text: &str, target_chars: usize) -> Vec<String> {
    let sentences = split_on_terminators(text);
    let mut out = Vec::new();
    for s in sentences {
        if s.chars().count() <= target_chars * 2 {
            out.push(s);
        } else {
            // Sentence too long for one cue. Split on commas / semicolons.
            for sub in split_on_clauses(&s, target_chars) {
                out.push(sub);
            }
        }
    }
    out
}

fn split_on_terminators(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut buf = String::new();
    for ch in text.chars() {
        buf.push(ch);
        if matches!(ch, '.' | '!' | '?') {
            // Only flush if the next non-space looks like a new sentence
            // (capital letter or end). Simpler: just flush. Edge cases
            // (e.g. "U.S.") will look odd but the worst outcome is a
            // single odd cue boundary, not broken output.
            let trimmed = buf.trim();
            if !trimmed.is_empty() {
                out.push(trimmed.to_string());
            }
            buf.clear();
        }
    }
    let tail = buf.trim();
    if !tail.is_empty() {
        out.push(tail.to_string());
    }
    out
}

fn split_on_clauses(sentence: &str, target_chars: usize) -> Vec<String> {
    let mut out = Vec::new();
    let mut buf = String::new();
    let mut buf_chars = 0;
    for ch in sentence.chars() {
        buf.push(ch);
        buf_chars += 1;
        if matches!(ch, ',' | ';') && buf_chars >= target_chars {
            out.push(buf.trim().to_string());
            buf.clear();
            buf_chars = 0;
        }
    }
    let tail = buf.trim();
    if !tail.is_empty() {
        out.push(tail.to_string());
    }
    if out.is_empty() {
        // Pathological: no commas, sentence is just very long. Hard-
        // split on word boundaries near the target char count.
        return hard_split_words(sentence, target_chars);
    }
    out
}

fn hard_split_words(sentence: &str, target_chars: usize) -> Vec<String> {
    let mut out = Vec::new();
    let mut buf = String::new();
    for word in sentence.split_whitespace() {
        if buf.chars().count() + word.chars().count() + 1 > target_chars && !buf.is_empty() {
            out.push(buf.trim().to_string());
            buf.clear();
        }
        if !buf.is_empty() {
            buf.push(' ');
        }
        buf.push_str(word);
    }
    let tail = buf.trim();
    if !tail.is_empty() {
        out.push(tail.to_string());
    }
    out
}

/// SRT format. Cues numbered from 1, timestamps `HH:MM:SS,mmm`,
/// blank line between cues.
pub fn render_srt(cues: &[SubtitleCue]) -> String {
    let mut out = String::new();
    for (i, cue) in cues.iter().enumerate() {
        out.push_str(&format!("{}\n", i + 1));
        out.push_str(&format_srt_time(cue.start_ms));
        out.push_str(" --> ");
        out.push_str(&format_srt_time(cue.end_ms));
        out.push('\n');
        out.push_str(&cue.text);
        out.push_str("\n\n");
    }
    out
}

/// VTT format. `WEBVTT` header, timestamps `HH:MM:SS.mmm`, blank line
/// between cues.
pub fn render_vtt(cues: &[SubtitleCue]) -> String {
    let mut out = String::from("WEBVTT\n\n");
    for cue in cues.iter() {
        out.push_str(&format_vtt_time(cue.start_ms));
        out.push_str(" --> ");
        out.push_str(&format_vtt_time(cue.end_ms));
        out.push('\n');
        out.push_str(&cue.text);
        out.push_str("\n\n");
    }
    out
}

fn format_srt_time(ms: u32) -> String {
    let total_s = ms / 1000;
    let h = total_s / 3600;
    let m = (total_s % 3600) / 60;
    let s = total_s % 60;
    let frac = ms % 1000;
    format!("{:02}:{:02}:{:02},{:03}", h, m, s, frac)
}

fn format_vtt_time(ms: u32) -> String {
    let total_s = ms / 1000;
    let h = total_s / 3600;
    let m = (total_s % 3600) / 60;
    let s = total_s % 60;
    let frac = ms % 1000;
    format!("{:02}:{:02}:{:02}.{:03}", h, m, s, frac)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_inputs_produce_empty_output() {
        assert!(cues_from_text("", 60_000).is_empty());
        assert!(cues_from_text("hello", 0).is_empty());
        assert!(cues_from_words(&[]).is_empty());
    }

    #[test]
    fn cues_span_full_duration() {
        let text = "First sentence. Second sentence. Third sentence.";
        let cues = cues_from_text(text, 30_000);
        assert!(!cues.is_empty());
        assert_eq!(cues.last().unwrap().end_ms, 30_000);
        assert_eq!(cues.first().unwrap().start_ms, 0);
    }

    #[test]
    fn srt_format_roundtrip() {
        let cues = vec![
            SubtitleCue {
                start_ms: 0,
                end_ms: 3500,
                text: "Hello there.".into(),
            },
            SubtitleCue {
                start_ms: 3500,
                end_ms: 7000,
                text: "How are you?".into(),
            },
        ];
        let srt = render_srt(&cues);
        assert!(srt.starts_with("1\n00:00:00,000 --> 00:00:03,500\nHello there."));
        assert!(srt.contains("\n\n2\n00:00:03,500 --> 00:00:07,000\nHow are you?"));
        assert!(srt.ends_with("\n\n"));
    }

    #[test]
    fn vtt_format_has_webvtt_header() {
        let cues = vec![SubtitleCue {
            start_ms: 1234,
            end_ms: 5678,
            text: "test".into(),
        }];
        let vtt = render_vtt(&cues);
        assert!(vtt.starts_with("WEBVTT\n\n"));
        assert!(vtt.contains("00:00:01.234 --> 00:00:05.678"));
    }

    #[test]
    fn time_format_handles_hours() {
        // 1h 23m 45.678s = 5025678 ms
        assert_eq!(format_srt_time(5_025_678), "01:23:45,678");
        assert_eq!(format_vtt_time(5_025_678), "01:23:45.678");
    }

    #[test]
    fn long_run_on_sentence_gets_split() {
        let long_sentence = "this is a very long run-on sentence that doesn't have any sentence terminators in it but does have some commas, like this one, and this one, and many more clauses, here are even more clauses, and a final clause to close it out";
        let cues = cues_from_text(long_sentence, 30_000);
        // Should be more than one cue because the long sentence got split.
        assert!(cues.len() > 1, "expected split, got {:?}", cues.len());
    }

    #[test]
    fn words_to_cues_breaks_on_pause() {
        let words = vec![
            WordTiming { text: "hello".into(), start_ms: 0,    end_ms: 500,  confidence: None },
            WordTiming { text: "world".into(), start_ms: 600,  end_ms: 1000, confidence: None },
            // 1000ms pause here
            WordTiming { text: "again".into(), start_ms: 2000, end_ms: 2400, confidence: None },
        ];
        let cues = cues_from_words(&words);
        assert_eq!(cues.len(), 2);
        assert_eq!(cues[0].text, "hello world");
        assert_eq!(cues[1].text, "again");
    }
}
