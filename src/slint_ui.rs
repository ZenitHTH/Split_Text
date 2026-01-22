slint::include_modules!();

pub fn decode_image_data(bytes: &[u8]) -> Result<(u32, u32, Vec<u8>), Box<dyn std::error::Error>> {
    let img = image::load_from_memory(bytes)?;
    let rgba = img.to_rgba8();
    Ok((rgba.width(), rgba.height(), rgba.into_raw()))
}

pub async fn fetch_and_update_thumbnail(
    video_id: String,
    ui_handle: slint::Weak<AppWindow>,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Scanning ID: {}", video_id);
    let url = format!("https://img.youtube.com/vi/{}/0.jpg", video_id);

    let response = reqwest::get(&url).await?;
    let img_bytes = response.bytes().await?;
    let (width, height, data) = decode_image_data(&img_bytes)?;

    let _ = slint::invoke_from_event_loop(move || {
        if let Some(ui) = ui_handle.upgrade() {
            let buffer = slint::SharedPixelBuffer::clone_from_slice(&data, width, height);
            let img = slint::Image::from_rgba8(buffer);
            ui.set_thumbnail_image(img);
        }
    });

    Ok(())
}
