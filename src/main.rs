mod tasks;

use file_spliter::split_file;
use std::env;

use std::process;
use tasks::{SplitMode, build_split_plan};
use youtube_subtitle_manager::{download_subtitle, extract_id, scan_subtitles};

slint::include_modules!();

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
            "Unknown command: '{}'. Use 'nth', 'manual', 'scan', 'download', or run without args for UI.",
            command
        )),
    }
}

// Helper to convert bytes (from reqwest) to slint::Image
// Helper to decode image to raw rgba
fn decode_image_data(bytes: &[u8]) -> Result<(u32, u32, Vec<u8>), Box<dyn std::error::Error>> {
    let img = image::load_from_memory(bytes)?;
    let rgba = img.to_rgba8();
    Ok((rgba.width(), rgba.height(), rgba.into_raw()))
}

#[tokio::main]
async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let mode = parse_args(&args).map_err(|e| e.to_string())?;

    match mode {
        AppMode::Ui => {
            let ui = AppWindow::new()?;
            let ui_handle = ui.as_weak();

            ui.on_request_scan(move |video_id| {
                let ui_handle = ui_handle.clone();
                let video_id = video_id.to_string();
                tokio::spawn(async move {
                    println!("Scanning ID: {}", video_id);
                    let id = extract_id(&video_id);
                    let url = format!("https://img.youtube.com/vi/{}/0.jpg", id);

                    match reqwest::get(&url).await {
                        Ok(response) => {
                            if let Ok(img_bytes) = response.bytes().await {
                                match decode_image_data(&img_bytes) {
                                    Ok((width, height, data)) => {
                                        let _ = slint::invoke_from_event_loop(move || {
                                            if let Some(ui) = ui_handle.upgrade() {
                                                let buffer =
                                                    slint::SharedPixelBuffer::clone_from_slice(
                                                        &data, width, height,
                                                    );
                                                let img = slint::Image::from_rgba8(buffer);
                                                ui.set_thumbnail_image(img);
                                            }
                                        });
                                    }
                                    Err(e) => eprintln!("Failed to decode image: {}", e),
                                }
                            } else {
                                eprintln!("Failed to get bytes from response");
                            }
                        }
                        Err(e) => eprintln!("Error fetching thumbnail: {}", e),
                    }
                });
            });

            ui.run()?;
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

            let filename = download_subtitle(&video_id, Some(lang)).await?;

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
