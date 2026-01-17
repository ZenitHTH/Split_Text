mod tasks;

use file_spliter::split_file;
use std::env;
use std::process;
use yt_subtitle_download::download_subtitle;
use tasks::{SplitMode, build_split_plan};

fn run() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        return Err(format!(
            "Usage:\n  Split:    {} <file> 1-100\n  Auto:     {} <file> -n 3000\n  Download: {} download <video_id> [lang_code]",
            args[0], args[0], args[0]
        ));
    }

    // --- FEATURE: DOWNLOAD SUBTITLE ---
    // ตรวจสอบคำสั่ง 'download' เพื่อเรียกใช้ youtube_downloader
    if args[1] == "download" {
        if args.len() < 3 {
            return Err("Usage: download <video_id_or_url> [lang_code] (default: en)".to_string());
        }
        let video_id = &args[2];
        // ถ้าไม่ระบุภาษา ให้ใช้ 'en' เป็นค่าเริ่มต้น
        let lang = if args.len() > 3 { &args[3] } else { "en" };

        // เรียกใช้ฟังก์ชัน download_subtitle จาก Library
        match download_subtitle(video_id, lang) {
            Ok(filename) => {
                println!("✅ Successfully saved subtitle to: {}", filename);
                return Ok(());
            }
            Err(e) => return Err(format!("Download failed: {}", e)),
        }
    }

    if args.len() < 3 {
        return Err(format!(
            "Usage:\n  Manual: {} <file> 1-100\n  Auto:   {} <file> -n 3000",
            args[0], args[0]
        ));
    }

    let input_path = args[1].clone();

    // 1. Determine the Mode
    let mode = if args[2] == "-n" {
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

    // 2. Build Plan (Single function call now!)
    let configs = build_split_plan(input_path.clone(), mode)?;

    println!("✅ Plan created: {} parts.", configs.len());

    // 3. Execute
    let success_msg = split_file(&input_path, &configs)?;

    println!("✅ {}", success_msg);
    Ok(())
}

fn main() {
    if let Err(e) = run() {
        eprintln!("\n❌ ERROR: {}", e);
        process::exit(1);
    }
}
