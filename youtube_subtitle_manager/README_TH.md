# YouTube Subtitle Manager

ไลบรารี Rust สำหรับจัดการซับไตเติ้ล YouTube ช่วยให้คุณสามารถดึงข้อมูลวิดีโอ, สแกนหาซับไตเติ้ลที่มี, และดาวน์โหลดออกมาเป็นไฟล์ SRT

## ฟีเจอร์ (Features)

- **Fetch Video Details**: ดึงชื่อคลิปและชื่อช่อง (Author)
- **Scan Subtitles**: ลิสต์รายการภาษาซับไตเติ้ลทั้งหมดที่มี (รวมถึงซับที่สร้างอัตโนมัติ)
- **Download Subtitles**: ดาวน์โหลดและแปลงซับไตเติ้ลให้อยู่ในรูปแบบ SRT พร้อม Timestamp ที่ถูกต้อง
- **ID Extraction**: ฟังก์ชันช่วยสำหรับดึง Video ID จาก URL รูปแบบต่างๆ

## การใช้งาน (Usage)

เพิ่ม Crate นี้ในโปรเจกต์ของคุณและเรียกใช้ฟังก์ชันต่างๆ ดังนี้:

```rust
use youtube_subtitle_manager::{fetch_video_details, scan_subtitles, download_subtitle};

#[tokio::main]
async fn main() {
    let video_id = "dQw4w9WgXcQ";

    // 1. ดึงข้อมูลวิดีโอ
    match fetch_video_details(video_id).await {
        Ok(details) => println!("Title: {}", details.title),
        Err(e) => eprintln!("Error: {}", e),
    }

    // 2. สแกนหาภาษาที่มี
    match scan_subtitles(video_id).await {
        Ok(list) => {
            for item in list {
                println!("Language: {} ({})", item.language, item.language_code);
            }
        }
        Err(e) => eprintln!("Error: {}", e),
    }

    // 3. ดาวน์โหลดซับภาษาอังกฤษ
    match download_subtitle(video_id, Some("en".to_string()), Some("output.srt".to_string())).await {
        Ok(path) => println!("Saved to: {}", path),
        Err(e) => eprintln!("Error: {}", e),
    }
}
```
