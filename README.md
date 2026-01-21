# YouTube Subtitle Manager & Splitter

A Rust-based command-line tool designed to manage and manipulate YouTube subtitles. It allows you to download subtitles from YouTube videos, scan for available languages, and split large subtitle files (SRT) into smaller chunks for processing.

## 📜 Origin / ที่มา

### English
Originally, this program was written to split subtitle files (SRT) for a streamer whose streams exceeded 4 hours in length. The goal was to take the split subtitle files and feed them into Google Gemini to generate accurate timestamps. However, I didn't have much time to write the code myself, so I mostly instructed Google Gemini to write it. Since I have some proficiency in Rust, the process involved directing the LLM and reviewing the code, rather than writing everything from scratch.

### Thai (ต้นฉบับ)
"เดิมทีโปรแกรมที่เขียนมาเพื่อแบ่งไฟล์ซับ (SRT) ของไลฟ์ สตีมเมอร์ท่านหนึ่งที่ไลฟ์ความยาวมากกว่า 4 ชม. แล้วนำไฟล์ซับที่แบ่งไปเข้า Google Gemini ในการทำ Timestamp แต่ฉันเวลาในการเขียนไม่ค่อยมี ส่วนมากจะเป็นการสั่ง Google Gemini เขียนโค้ด แต่ฉันเขียน Rust-Lang เป็นระดับหนึ่ง เลยจะเป็นการสั่ง LLM แล้วตรวจโค้ดซะมากกว่าการเขียนจริงๆ"

## ✨ Features

- **GUI Mode**: A graphical user interface to easily interact with the tool, including displaying YouTube video thumbnails.
- **Scan Subtitles**: List all available subtitle languages for a specific YouTube video.
- **Download Subtitles**: Download subtitles (SRT format) for a specific YouTube video.
- **Split File (Auto)**: Split a large file into smaller chunks based on a fixed number of lines.
- **Split File (Manual)**: Split a file based on specific line ranges.

## 🚀 Usage

Run the program using `cargo run` followed by the command and arguments.

### 0. Launch GUI
Run without arguments to open the Graphical User Interface.
```bash
cargo run
```
The GUI now allows scanning video IDs and displays the video thumbnail!

### 1. Scan for Subtitles
Check what languages are available for a video.
```bash
cargo run -- scan <video_id_or_url>
```

### 2. Download Subtitles
Download the subtitle file. Default language is English (`en`).
```bash
cargo run -- download <video_id_or_url> [lang]
```
Example:
```bash
cargo run -- download dQw4w9WgXcQ en
```

### 3. Split File (Nth / Auto)
Split a file into chunks of a specific size (number of lines).
```bash
cargo run -- nth <file_path> <lines_per_chunk>
```
Example:
```bash
cargo run -- nth subtitles.srt 1000
```

### 4. Split File (Manual)
Split specific ranges of lines from a file.
```bash
cargo run -- manual <file_path> <range1> <range2> ...
```
Example (Split lines 1-100 and 200-300):
```bash
cargo run -- manual subtitles.srt 1-100 200-300
```

## 🛠️ Build

To build the project for release:

```bash
cargo build --release
```

The binary will be located in `target/release/`.
