slint::include_modules!();
use crate::tasks::{self, SplitMode};
use file_spliter::split_file;
use slint::{ComponentHandle, Image, SharedPixelBuffer, SharedString, VecModel};
use std::rc::Rc;
use youtube_subtitle_manager::{
    download_subtitle, extract_id, fetch_video_details, scan_subtitles,
};

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
    // 1. Fetch Thumbnail Data
    let thumbnail_result = fetch_thumbnail_data(&video_id).await;

    // 2. Handle Thumbnail / Validity Result
    match thumbnail_result {
        Ok((width, height, data)) => {
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
        Err(_) => {
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
            return; // Stop processing if invalid
        }
    }

    // 3. Fetch Metadata & Subtitles Concurrenty
    let (meta_result, subtitles_result) =
        tokio::join!(fetch_video_details(&video_id), scan_subtitles(&video_id));

    // 4. Update UI with Details
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

pub fn run_ui() -> Result<(), Box<dyn std::error::Error>> {
    let ui = AppWindow::new()?;
    let ui_handle = ui.as_weak();

    ui.on_check_link(move |url| {
        let video_id = extract_id(&url).to_string();
        let ui_handle = ui_handle.clone();
        tokio::spawn(async move {
            check_video_status(video_id, ui_handle).await;
        });
    });

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

    // --- Split Text Callbacks ---
    setup_split_handlers(&ui);

    ui.run()?;
    Ok(())
}

fn setup_split_handlers(ui: &AppWindow) {
    setup_file_picker_handler(ui);
    setup_folder_picker_handler(ui);
    setup_execute_split_handler(ui);
}

fn setup_file_picker_handler(ui: &AppWindow) {
    let ui_handle = ui.as_weak();
    ui.on_pick_split_file(move || {
        let ui_handle = ui_handle.clone();
        tokio::spawn(async move {
            if let Some(file) = rfd::AsyncFileDialog::new()
                .set_title("Select File to Split")
                .pick_file()
                .await
            {
                let path = file.path().to_string_lossy().to_string();
                let _ = slint::invoke_from_event_loop(move || {
                    if let Some(ui) = ui_handle.upgrade() {
                        ui.set_split_input_path(path.into());
                    }
                });
            }
        });
    });
}

fn setup_folder_picker_handler(ui: &AppWindow) {
    let ui_handle = ui.as_weak();
    ui.on_pick_output_folder(move || {
        let ui_handle = ui_handle.clone();
        tokio::spawn(async move {
            if let Some(folder) = rfd::AsyncFileDialog::new()
                .set_title("Select Output Directory")
                .pick_folder()
                .await
            {
                let path = folder.path().to_string_lossy().to_string();
                let _ = slint::invoke_from_event_loop(move || {
                    if let Some(ui) = ui_handle.upgrade() {
                        ui.set_split_output_path(path.into());
                    }
                });
            }
        });
    });
}

fn setup_execute_split_handler(ui: &AppWindow) {
    let ui_handle = ui.as_weak();
    ui.on_execute_split(move |input_path, output_path, mode_index, param| {
        let input_path = input_path.to_string();
        let output_path = if output_path.is_empty() {
            None
        } else {
            Some(output_path.to_string())
        };
        let param = param.to_string();
        let ui_handle = ui_handle.clone();

        tokio::spawn(async move {
            // Determine logic based on mode_index (0 = Auto, 1 = Manual)
            let result = async {
                let mode = if mode_index == 0 {
                    let size = param
                        .parse::<usize>()
                        .map_err(|_| "Invalid chunk size: must be a positive number")?;
                    SplitMode::Auto {
                        chunk_size: size,
                        output_dir: output_path,
                    }
                } else {
                    let ranges: Vec<String> =
                        param.split_whitespace().map(|s| s.to_string()).collect();
                    if ranges.is_empty() {
                        return Err("No ranges provided".into());
                    }
                    SplitMode::Manual {
                        ranges,
                        output_dir: output_path,
                    }
                };

                // 1. Build Plan
                let configs = tasks::build_split_plan(input_path.clone(), mode)?;

                // 2. Execute Split
                split_file(&input_path, &configs)?;

                Ok::<String, String>(format!("Successfully split into {} parts.", configs.len()))
            }
            .await;

            // Update UI
            let _ = slint::invoke_from_event_loop(move || {
                if let Some(ui) = ui_handle.upgrade() {
                    match result {
                        Ok(msg) => {
                            ui.set_split_status_message(msg.into());
                            ui.set_split_status_color(slint::Color::from_rgb_u8(0, 150, 0)); // Greenish
                        }
                        Err(e) => {
                            ui.set_split_status_message(format!("Error: {}", e).into());
                            ui.set_split_status_color(slint::Color::from_rgb_u8(255, 0, 0));
                        }
                    }
                }
            });
        });
    });
}
