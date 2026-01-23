# YouTube Subtitle Manager

A Rust library for interacting with YouTube subtitles. It allows you to fetch video details, list available transcripts, and download subtitles in SRT format.

This crate leverages **[yt-transcript-rs](https://crates.io/crates/yt-transcript-rs)** to handle low-level interactions with YouTube's internal APIs.

## Features

- **Fetch Video Details**: Get the title and author of a YouTube video.
- **Scan Subtitles**: List all available subtitle languages (including auto-generated ones).
- **Download Subtitles**: Download subtitles and convert them to standard SRT format with timestamps.
- **ID Extraction**: Robust utility to extract YouTube Video IDs from various URL formats.

## Usage

Add this crate to your project and use the provided functions:

```rust
use youtube_subtitle_manager::{fetch_video_details, scan_subtitles, download_subtitle};

#[tokio::main]
async fn main() {
    let video_id = "dQw4w9WgXcQ";

    // 1. Fetch Details
    match fetch_video_details(video_id).await {
        Ok(details) => println!("Title: {}", details.title),
        Err(e) => eprintln!("Error: {}", e),
    }

    // 2. Scan available languages
    match scan_subtitles(video_id).await {
        Ok(list) => {
            for item in list {
                println!("Language: {} ({})", item.language, item.language_code);
            }
        }
        Err(e) => eprintln!("Error: {}", e),
    }

    // 3. Download English subtitles
    match download_subtitle(video_id, Some("en".to_string()), Some("output.srt".to_string())).await {
        Ok(path) => println!("Saved to: {}", path),
        Err(e) => eprintln!("Error: {}", e),
    }
}
```
