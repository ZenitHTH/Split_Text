use super::AppWindow;
use crate::tasks::{self, SplitMode};
use file_spliter::split_file;
use slint::ComponentHandle;

pub fn setup_handlers(ui: &AppWindow) {
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

/// Executes the split logic based on the provided mode and parameters.
async fn process_split_task(
    input_path: String,
    output_path: Option<String>,
    mode_index: i32,
    param: String,
) -> Result<String, String> {
    let mode = if mode_index == 0 {
        let size = param
            .parse::<usize>()
            .map_err(|_| "Invalid chunk size: must be a positive number")?;
        SplitMode::Auto {
            chunk_size: size,
            output_dir: output_path,
        }
    } else {
        let ranges: Vec<String> = param.split_whitespace().map(|s| s.to_string()).collect();
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

    Ok(format!("Successfully split into {} parts.", configs.len()))
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
            let result = process_split_task(input_path, output_path, mode_index, param).await;

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
