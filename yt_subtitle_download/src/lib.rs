mod models;

use reqwest::blocking::Client;
use regex::Regex;
use std::fmt::Write; // Import for writeln! macro
use anyhow::{Result, Context, anyhow};
use models::*;

// --- Logic ---

/// Extracts the ytInitialPlayerResponse JSON from the HTML using Regex
fn extract_player_response(html: &str) -> Result<PlayerResponse> {
    // Compile regex to find the player response object
    let re = Regex::new(r"var ytInitialPlayerResponse\s*=\s*(\{.*?\});")?;
    
    // Capture the JSON string
    let captures = re.captures(html)
        .context("Could not find ytInitialPlayerResponse in HTML")?;
    let json_str = captures.get(1)
        .context("Could not capture JSON group")?
        .as_str();

    // Deserialize into our struct
    let response: PlayerResponse = serde_json::from_str(json_str)
        .context("Failed to deserialize PlayerResponse")?;
    
    Ok(response)
}

/// Converts json3 events to standard SRT format safely
fn events_to_srt(events: Vec<TranscriptEvent>) -> Result<String> {
    let mut output = String::new();
    let mut counter = 1;

    for event in events {
        // Ensure we have start time, duration, and segments
        if let (Some(start), Some(duration), Some(segs)) = (event.t_start_ms, event.d_duration_ms, event.segs) {
            let end = start + duration;
            
            // Combine all text segments into one string
            let raw_text: String = segs.iter().map(|s| s.utf8.clone()).collect();
            
            // Decode HTML entities (e.g., &#39; -> ')
            let clean_text = html_escape::decode_html_entities(&raw_text);

            let start_ts = format_timestamp(start);
            let end_ts = format_timestamp(end);

            // Write to buffer, propagating errors if writing fails
            writeln!(&mut output, "{}", counter)?;
            writeln!(&mut output, "{} --> {}", start_ts, end_ts)?;
            writeln!(&mut output, "{}\n", clean_text)?;
            
            counter += 1;
        }
    }
    Ok(output)
}

/// Formats milliseconds into SRT timestamp format (HH:MM:SS,ms)
fn format_timestamp(ms: u64) -> String {
    let seconds = ms / 1000;
    let milliseconds = ms % 1000;
    let minutes = seconds / 60;
    let hours = minutes / 60;
    format!("{:02}:{:02}:{:02},{:03}", hours, minutes % 60, seconds % 60, milliseconds)
}

/// Public Entrypoint: Downloads and saves the subtitle
pub fn download_subtitle(video_id_or_url: &str, lang: &str) -> Result<String> {
    // 1. Extract Video ID safely
    let video_id = if video_id_or_url.contains("v=") {
        video_id_or_url
            .split("v=")
            .nth(1)
            .ok_or_else(|| anyhow!("Invalid URL: Could not find 'v=' parameter"))?
            .split('&')
            .next()
            .ok_or_else(|| anyhow!("Invalid URL: Video ID segment is empty"))?
    } else {
        video_id_or_url
    };

    println!("⬇️  Fetching metadata for Video ID: {}", video_id);

    // 2. Build Client (Mimic Browser to avoid bot detection)
    let client = Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .build()?;

    // 3. Fetch Video Page
    let url = format!("https://www.youtube.com/watch?v={}", video_id);
    let html = client.get(&url).send()?.text()?;

    // 4. Extract Metadata & Captions
    let player_data = extract_player_response(&html)?;
    let video_title = player_data.video_details
        .map(|d| d.title)
        .unwrap_or_else(|| "video".to_string());

    let captions = player_data.captions
        .context("No captions found for this video. (Check if the video actually has CC enabled)")?;

    // 5. Find the requested language
    println!("🔍 Searching for language: '{}'...", lang);
    let track = captions.renderer.tracks.iter()
        .find(|t| t.lang_code == lang)
        .ok_or_else(|| {
            let available_langs: Vec<&String> = captions.renderer.tracks.iter().map(|t| &t.lang_code).collect();
            anyhow!("Language '{}' not available. Available: {:?}", lang, available_langs)
        })?;

    // 6. Fetch Transcript (json3 format)
    let transcript_url = format!("{}&fmt=json3", track.base_url);
    let transcript_json = client.get(&transcript_url).send()?.text()?;
    
    // 7. Deserialize and Convert to SRT
    let transcript: TranscriptResponse = serde_json::from_str(&transcript_json)?;
    let srt_content = events_to_srt(transcript.events)?;

    // 8. Save to File
    let safe_title = video_title.replace(|c: char| !c.is_alphanumeric() && c != ' ', "").replace(' ', "_");
    let filename = format!("{}_{}.srt", safe_title, lang);
    std::fs::write(&filename, srt_content)?;

    Ok(filename)
}