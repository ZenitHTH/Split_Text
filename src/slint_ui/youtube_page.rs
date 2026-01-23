use super::{AppWindow, SubtitleItem};
use slint::{ComponentHandle, Image, SharedPixelBuffer, SharedString, VecModel};
use std::rc::Rc;
use youtube_subtitle_manager::{
    download_subtitle, extract_id, fetch_video_details, scan_subtitles,
};

pub fn setup_handlers(ui: &AppWindow) {
    setup_check_link_handler(ui);
    setup_save_location_handler(ui);
    setup_download_subtitle_handler(ui);
}

/// Decodes image bytes into raw RGBA data.
pub fn decode_image_data(
    bytes: &[u8],
) -> Result<(u32, u32, Vec<u8>), Box<dyn std::error::Error + Send + Sync>> {
    let img = image::load_from_memory(bytes)?;
    let rgba = img.to_rgba8();
    Ok((rgba.width(), rgba.height(), rgba.into_raw()))
}

/// Fetches the YouTube video thumbnail and returns raw data.
async fn fetch_thumbnail_data(
    video_id: &str,
) -> Result<(u32, u32, Vec<u8>), Box<dyn std::error::Error + Send + Sync>> {
    let url = format!("https://img.youtube.com/vi/{}/0.jpg", video_id);
    let response = reqwest::get(&url).await?;

    if !response.status().is_success() {
        return Err("Failed to fetch thumbnail".into());
    }

    let bytes = response.bytes().await?;
    decode_image_data(&bytes)
}

/// Orchestrates checking video status, fetching metadata, and updating the UI.
pub async fn check_video_status(video_id: String, ui_handle: slint::Weak<AppWindow>) {
    if verify_video_thumbnail(&video_id, &ui_handle).await {
        fetch_and_update_details(&video_id, &ui_handle).await;
    }
}

/// Verifies the video by checking its thumbnail.
/// Returns true if valid, false otherwise.
async fn verify_video_thumbnail(video_id: &str, ui_handle: &slint::Weak<AppWindow>) -> bool {
    let thumbnail_result = fetch_thumbnail_data(video_id).await;

    match thumbnail_result {
        Ok((width, height, data)) => {
            handle_thumbnail_success(ui_handle, width, height, data);
            true
        }
        Err(_) => {
            handle_thumbnail_failure(ui_handle);
            false
        }
    }
}

fn handle_thumbnail_success(
    ui_handle: &slint::Weak<AppWindow>,
    width: u32,
    height: u32,
    data: Vec<u8>,
) {
    let ui_weak = ui_handle.clone();
    let _ = slint::invoke_from_event_loop(move || {
        if let Some(ui) = ui_weak.upgrade() {
            let buffer = SharedPixelBuffer::clone_from_slice(&data, width, height);
            let img = Image::from_rgba8(buffer);
            ui.set_thumbnail_image(img);
            ui.set_status_message("Valid YouTube Link".into());
            ui.set_status_color(slint::Color::from_rgb_u8(0, 255, 0));
        }
    });
}

fn handle_thumbnail_failure(ui_handle: &slint::Weak<AppWindow>) {
    let ui_weak = ui_handle.clone();
    let _ = slint::invoke_from_event_loop(move || {
        if let Some(ui) = ui_weak.upgrade() {
            ui.set_status_message("Invalid Link or Network Error".into());
            ui.set_status_color(slint::Color::from_rgb_u8(255, 0, 0));
            // Reset fields
            ui.set_video_title("".into());
            ui.set_video_author("".into());
            ui.set_subtitle_list(Rc::new(VecModel::default()).into());
        }
    });
}

/// Fetches and updates metadata and subtitles.
async fn fetch_and_update_details(video_id: &str, ui_handle: &slint::Weak<AppWindow>) {
    // Fetch Metadata & Subtitles Concurrenty
    let (meta_result, subtitles_result) =
        tokio::join!(fetch_video_details(video_id), scan_subtitles(video_id));

    // Update UI with Details
    let ui_weak = ui_handle.clone();
    let _ = slint::invoke_from_event_loop(move || {
        if let Some(ui) = ui_weak.upgrade() {
            // Update Metadata
            match meta_result {
                Ok(details) => {
                    ui.set_video_title(details.title.into());
                    ui.set_video_author(details.author.into());
                }
                Err(_) => {
                    ui.set_video_title("Unknown Title".into());
                    ui.set_video_author("Unknown Channel".into());
                }
            }

            // Update Subtitle List
            match subtitles_result {
                Ok(subs) => {
                    let slint_subs: Vec<SubtitleItem> = subs
                        .into_iter()
                        .map(|s| SubtitleItem {
                            code: SharedString::from(s.language_code),
                            name: SharedString::from(s.language),
                            is_generated: s.is_generated,
                        })
                        .collect();
                    ui.set_subtitle_list(Rc::new(VecModel::from(slint_subs)).into());
                }
                Err(_) => {
                    ui.set_subtitle_list(Rc::new(VecModel::default()).into());
                }
            }
        }
    });
}

fn setup_check_link_handler(ui: &AppWindow) {
    let ui_handle = ui.as_weak();
    ui.on_check_link(move |url| {
        let video_id = extract_id(&url).to_string();
        let ui_handle = ui_handle.clone();
        tokio::spawn(async move {
            check_video_status(video_id, ui_handle).await;
        });
    });
}

fn setup_save_location_handler(ui: &AppWindow) {
    let ui_handle = ui.as_weak();
    ui.on_select_save_location(move |url| {
        let video_id = extract_id(&url).to_string();
        let ui_handle = ui_handle.clone();
        tokio::spawn(async move {
            let default_name = format!("{}.srt", video_id);
            if let Some(file) = rfd::AsyncFileDialog::new()
                .add_filter("Subtitle", &["srt"])
                .set_file_name(&default_name)
                .save_file()
                .await
            {
                let path = file.path().to_string_lossy().to_string();
                let _ = slint::invoke_from_event_loop(move || {
                    if let Some(ui) = ui_handle.upgrade() {
                        ui.set_save_path(path.into());
                    }
                });
            }
        });
    });
}

fn setup_download_subtitle_handler(ui: &AppWindow) {
    let ui_handle = ui.as_weak();
    ui.on_download_subtitle(move |url, path, lang| {
        let video_id = extract_id(&url).to_string();
        let path = path.to_string();
        let lang = lang.to_string();
        let ui_handle = ui_handle.clone();
        tokio::spawn(async move {
            match download_subtitle(&video_id, Some(lang), Some(path)).await {
                Ok(_) => {
                    let _ = slint::invoke_from_event_loop(move || {
                        if let Some(ui) = ui_handle.upgrade() {
                            ui.set_status_message("Download Successful".into());
                            ui.set_status_color(slint::Color::from_rgb_u8(0, 255, 0));
                        }
                    });
                }
                Err(e) => {
                    let msg = format!("Error: {}", e);
                    let _ = slint::invoke_from_event_loop(move || {
                        if let Some(ui) = ui_handle.upgrade() {
                            ui.set_status_message(msg.into());
                            ui.set_status_color(slint::Color::from_rgb_u8(255, 0, 0));
                        }
                    });
                }
            }
        });
    });
}
