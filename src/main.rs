mod tasks;

use file_spliter::split_file;
use std::env;
use std::process;
use yt_subtitle_download::download_subtitle;
use tasks::{SplitMode, build_split_plan};

enum AppMode {
    Download { video_id: String, lang: String },
    Split { input_path: String, mode: SplitMode },
}

fn parse_args(args: &[String]) -> Result<AppMode, String> {
    if args.len() < 2 {
        return Err(format!(
            "Usage:\n  Split:    {} <file> 1-100\n  Auto:     {} <file> -n 3000\n  Download: {} download <video_id> [lang_code]",
            args[0], args[0], args[0]
        ));
    }

    if args[1] == "download" {
        if args.len() < 3 {
            return Err("Usage: download <video_id_or_url> [lang_code] (default: en)".to_string());
        }
        let video_id = args[2].clone();
        let lang = if args.len() > 3 { args[3].clone() } else { "en".to_string() };
        return Ok(AppMode::Download { video_id, lang });
    }

    if args.len() < 3 {
        return Err(format!(
            "Usage:\n  Manual: {} <file> 1-100\n  Auto:   {} <file> -n 3000",
            args[0], args[0]
        ));
    }

    let input_path = args[1].clone();
    let split_mode = if args[2] == "-n" {
        if args.len() < 4 {
            return Err("Missing count for -n".to_string());
        }
        let size = args[3].parse::<usize>().map_err(|_| "Invalid number")?;
        println!("🔄 Auto-Mode selected ({} lines/chunk)", size);
        SplitMode::Auto(size)
    } else {
        println!("🔧 Manual-Mode selected");
        let ranges = args[2..].to_vec();
        SplitMode::Manual(ranges)
    };

    Ok(AppMode::Split { input_path, mode: split_mode })
}

fn run() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();
    let mode = parse_args(&args)?;

    match mode {
        AppMode::Download { video_id, lang } => {
            match download_subtitle(&video_id, &lang) {
                Ok(filename) => {
                    println!("✅ Successfully saved subtitle to: {}", filename);
                    Ok(())
                }
                Err(e) => Err(format!("Download failed: {}", e)),
            }
        }
        AppMode::Split { input_path, mode } => {
            let configs = build_split_plan(input_path.clone(), mode)?;
            println!("✅ Plan created: {} parts.", configs.len());
            let success_msg = split_file(&input_path, &configs)?;
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
