use std::fs::File;
use std::io::Write;
use thiserror::Error;
use yt_transcript_rs::YouTubeTranscriptApi;

#[derive(Error, Debug)]
pub enum SubtitleError {
    #[error("YouTube transcript error: {0}")]
    TranscriptError(#[source] Box<dyn std::error::Error + Send + Sync>),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Process error: {0}")]
    Other(String),
}

pub struct TranscriptInfo {
    pub language_code: String,
    pub language: String,
    pub is_generated: bool,
}

pub fn extract_id(input: &str) -> &str {
    if input.contains("v=") {
        input
            .split("v=")
            .nth(1)
            .and_then(|s| s.split('&').next())
            .unwrap_or(input)
    } else {
        input
    }
}

fn format_timestamp(seconds: f64) -> String {
    let total_ms = (seconds * 1000.0) as u64;
    let s = total_ms / 1000;
    let ms = total_ms % 1000;
    let m = s / 60;
    let h = m / 60;
    format!("{:02}:{:02}:{:02},{:03}", h, m % 60, s % 60, ms)
}

pub async fn scan_subtitles(video_id: &str) -> Result<Vec<TranscriptInfo>, SubtitleError> {
    let id = extract_id(video_id);
    let api = YouTubeTranscriptApi::new(None, None, None)
        .map_err(|e| SubtitleError::TranscriptError(Box::new(e)))?;

    let transcripts = api
        .list_transcripts(id)
        .await
        .map_err(|e| SubtitleError::TranscriptError(Box::new(e)))?;

    let mut infos = Vec::new();
    for t in transcripts.transcripts() {
        infos.push(TranscriptInfo {
            language_code: t.language_code().to_string(),
            language: t.language().to_string(),
            is_generated: t.is_generated(),
        });
    }

    Ok(infos)
}

pub async fn download_subtitle(
    video_id: &str,
    lang: Option<String>,
) -> Result<String, SubtitleError> {
    let id = extract_id(video_id);
    let lang_code = lang.unwrap_or_else(|| "en".to_string());

    let api = YouTubeTranscriptApi::new(None, None, None)
        .map_err(|e| SubtitleError::TranscriptError(Box::new(e)))?;

    let transcript = api
        .fetch_transcript(id, &[&lang_code], false)
        .await
        .map_err(|e| SubtitleError::TranscriptError(Box::new(e)))?;

    let filename = format!("{}_{}.srt", id, lang_code);
    let mut file = File::create(&filename)?;

    let mut counter = 1;
    for part in transcript.parts() {
        let start = part.start;
        let duration = part.duration;
        let end = start + duration;
        let text = &part.text;

        writeln!(file, "{}", counter)?;
        writeln!(
            file,
            "{} --> {}",
            format_timestamp(start),
            format_timestamp(end)
        )?;
        writeln!(file, "{}\n", text)?;
        counter += 1;
    }

    Ok(filename)
}
