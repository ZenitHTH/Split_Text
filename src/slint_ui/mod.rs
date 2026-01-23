slint::include_modules!();

mod split_page;
mod youtube_page;

pub fn run_ui() -> Result<(), Box<dyn std::error::Error>> {
    let ui = AppWindow::new()?;

    youtube_page::setup_handlers(&ui);
    split_page::setup_handlers(&ui);

    ui.run()?;
    Ok(())
}
