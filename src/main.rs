mod tasks;

use file_spliter::split_file;
use std::env;
use std::io::Write;
use std::process;
use tasks::{SplitMode, build_split_plan};
use yt_transcript_rs::YouTubeTranscriptApi;

enum AppMode {
    Download { video_id: String, lang: String },
    Scan { video_id: String },
    Split { input_path: String, mode: SplitMode },
    Help,
}

fn print_usage(program_name: &str) {
    println!("Usage:");
    println!(
        "  nth      {} <file> <size>     | Split file into chunks of <size> lines",
        program_name
    );
    println!(
        "  manual   {} <file> <range>... | Split specific ranges (e.g. 1-100 200-300)",
        program_name
    );
    println!(
        "  scan     {} <video_id>        | List available subtitle languages",
        program_name
    );
    println!(
        "  download {} <video_id> [lang] | Download YouTube subtitles (default lang: en)",
        program_name
    );
    println!(
        "  help     {}                   | Show this help message",
        program_name
    );
}

fn parse_args(args: &[String]) -> Result<AppMode, String> {
    if args.len() < 2 {
        return Ok(AppMode::Help);
    }

    let command = args[1].as_str();

    match command {
        "help" => Ok(AppMode::Help),
        "scan" => {
            if args.len() < 3 {
                return Err("Usage: scan <video_id_or_url>".to_string());
            }
            Ok(AppMode::Scan {
                video_id: args[2].clone(),
            })
        }
        "download" => {
            if args.len() < 3 {
                return Err("Usage: download <video_id_or_url> [lang]".to_string());
            }
            let video_id = args[2].clone();
            let lang = if args.len() > 3 {
                args[3].clone()
            } else {
                "en".to_string()
            };
            Ok(AppMode::Download { video_id, lang })
        }
        "nth" => {
            if args.len() < 4 {
                return Err("Usage: nth <file> <size>".to_string());
            }
            let input_path = args[2].clone();
            let size = args[3]
                .parse::<usize>()
                .map_err(|_| "Invalid chunk size number")?;
            println!("🔄 'nth' (Auto) Mode selected ({} lines/chunk)", size);
            Ok(AppMode::Split {
                input_path,
                mode: SplitMode::Auto(size),
            })
        }
        "manual" => {
            if args.len() < 4 {
                return Err("Usage: manual <file> <range>...".to_string());
            }
            let input_path = args[2].clone();
            let ranges = args[3..].to_vec();
            println!("🔧 'manual' Mode selected");
            Ok(AppMode::Split {
                input_path,
                mode: SplitMode::Manual(ranges),
            })
        }
        _ => Err(format!(
            "Unknown command: '{}'. Use 'nth', 'manual', 'scan', or 'download'.",
            command
        )),
    }
}

fn extract_id(input: &str) -> &str {
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

#[tokio::main]
async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let mode = parse_args(&args).map_err(|e| e.to_string())?;

    match mode {
        AppMode::Help => {
            print_usage(&args[0]);
            Ok(())
        }
        AppMode::Scan { video_id } => {
            let id = extract_id(&video_id);
            println!("🔍 Scanning subtitles for ID: {}", id);

            let api = YouTubeTranscriptApi::new(None, None, None)?;
            let transcripts = api.list_transcripts(id).await?;

            println!("{:<10} | {:<25} | {:<10}", "Code", "Language", "Type");
            println!("{:-<10}-+-{:-<25}-+-{:-<10}", "", "", "");

            for t in transcripts.transcripts() {
                let kind = if t.is_generated() { "Auto" } else { "Manual" };
                println!(
                    "{:<10} | {:<25} | {:<10}",
                    t.language_code(),
                    t.language(),
                    kind
                );
            }
            Ok(())
        }
        AppMode::Download { video_id, lang } => {
            let id = extract_id(&video_id);
            println!("⬇️  Downloading subtitle for ID: {} (Lang: {})", id, lang);

            let api = YouTubeTranscriptApi::new(None, None, None)?;

            // Try to fetch the transcript
            let transcript = api.fetch_transcript(id, &[&lang], false).await?;

            // Generate Filename
            // Note: This lib doesn't fetch video metadata (title) by default in fetch_transcript.
            // We use the ID for the filename to be safe, or you could call fetch_video_details if needed.
            let filename = format!("{}_{}.srt", id, lang);
            let mut file = std::fs::File::create(&filename)?;

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

            println!("✅ Successfully saved subtitle to: {}", filename);
            Ok(())
        }
        AppMode::Split { input_path, mode } => {
            let configs = build_split_plan(input_path.clone(), mode).map_err(|e| e.to_string())?;
            println!("✅ Plan created: {} parts.", configs.len());
            let success_msg = split_file(&input_path, &configs).map_err(|e| e.to_string())?;
            println!("✅ {}", success_msg);
            Ok(())
        }
    }
}

fn main() {
    if let Err(e) = run() {
        eprintln!("\n❌ ERROR: {}", e);
        process::exit(1);
    }
}
