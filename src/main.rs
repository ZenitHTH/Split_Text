mod slint_ui;
mod tasks;

use file_spliter::split_file;
use std::env;

use std::process;
use tasks::{SplitMode, build_split_plan};
use youtube_subtitle_manager::{download_subtitle, extract_id, scan_subtitles};

// slint imports removed as they are now handled in slint_ui.rs

enum AppMode {
    Ui,
    Download { video_id: String, lang: String },
    Scan { video_id: String },
    Split { input_path: String, mode: SplitMode },
    Help,
}

fn print_usage(program_name: &str) {
    println!("Usage:");
    println!("  (no args)                        | Launch GUI mode",);
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
        return Ok(AppMode::Ui);
    }

    let command = args[1].as_str();

    match command {
        "ui" => Ok(AppMode::Ui),
        "help" | "--help" | "-h" => Ok(AppMode::Help),
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
                mode: SplitMode::Auto {
                    chunk_size: size,
                    output_dir: None,
                },
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
                mode: SplitMode::Manual {
                    ranges,
                    output_dir: None,
                },
            })
        }
        _ => Err(format!(
            "Unknown command: '{}'. Use 'nth', 'manual', 'scan', 'download', or run without args for UI.",
            command
        )),
    }
}

#[tokio::main]
async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let mode = parse_args(&args).map_err(|e| e.to_string())?;

    match mode {
        AppMode::Ui => {
            slint_ui::run_ui()?;
            Ok(())
        }
        AppMode::Help => {
            print_usage(&args[0]);
            Ok(())
        }
        AppMode::Scan { video_id } => {
            let id = extract_id(&video_id);
            println!("🔍 Scanning subtitles for ID: {}", id);

            let transcripts = scan_subtitles(&video_id).await?;

            println!("{:<10} | {:<25} | {:<10}", "Code", "Language", "Type");
            println!("{:-<10}-+-{:-<25}-+-{:-<10}", "", "", "");

            for t in transcripts {
                let kind = if t.is_generated { "Auto" } else { "Manual" };
                println!(
                    "{:<10} | {:<25} | {:<10}",
                    t.language_code, t.language, kind
                );
            }
            Ok(())
        }
        AppMode::Download { video_id, lang } => {
            let id = extract_id(&video_id);
            println!("⬇️  Downloading subtitle for ID: {} (Lang: {})", id, lang);

            let filename = download_subtitle(&video_id, Some(lang), None).await?;

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
