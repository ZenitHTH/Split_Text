mod tasks;

use file_spliter::split_file;
use std::env;
use std::process;
use yt_subtitle_download::download_subtitle;
use tasks::{SplitMode, build_split_plan};

enum AppMode {
    Download { video_id: String, lang: String },
    Split { input_path: String, mode: SplitMode },
    Help,
}

fn print_usage(program_name: &str) {
    println!("Usage:");
    println!("  nth      {} <file> <size>     | Split file into chunks of <size> lines", program_name);
    println!("  manual   {} <file> <range>... | Split specific ranges (e.g. 1-100 200-300)", program_name);
    println!("  download {} <video_id> [lang] | Download YouTube subtitles (default lang: en)", program_name);
    println!("  help     {}                   | Show this help message", program_name);
}

fn parse_args(args: &[String]) -> Result<AppMode, String> {
    if args.len() < 2 {
        return Ok(AppMode::Help);
    }

    let command = args[1].as_str();

    match command {
        "help" => Ok(AppMode::Help),
        "download" => {
            if args.len() < 3 {
                return Err("Usage: download <video_id> [lang]".to_string());
            }
            let video_id = args[2].clone();
            let lang = if args.len() > 3 { args[3].clone() } else { "en".to_string() };
            Ok(AppMode::Download { video_id, lang })
        }
        "nth" => {
            if args.len() < 4 {
                return Err("Usage: nth <file> <size>".to_string());
            }
            let input_path = args[2].clone();
            let size = args[3].parse::<usize>().map_err(|_| "Invalid chunk size number")?;
            println!("🔄 'nth' (Auto) Mode selected ({} lines/chunk)", size);
            Ok(AppMode::Split { input_path, mode: SplitMode::Auto(size) })
        }
        "manual" => {
            if args.len() < 4 {
                return Err("Usage: manual <file> <range>...".to_string());
            }
            let input_path = args[2].clone();
            let ranges = args[3..].to_vec();
            println!("🔧 'manual' Mode selected");
            Ok(AppMode::Split { input_path, mode: SplitMode::Manual(ranges) })
        }
        _ => Err(format!("Unknown command: '{}'. Use 'nth', 'manual', or 'download'.", command)),
    }
}

fn run() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();
    let mode = parse_args(&args)?;

    match mode {
        AppMode::Help => {
            print_usage(&args[0]);
            Ok(())
        }
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
