/*
    Widgets we need:
      - Hex Viewer
      - Packet List
      - Filter Bar
      - Main Window
      - Connect Window
      - Menu Bar?

    ConnectionState: Connected / Not Connected
*/
mod app;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let native_options = eframe::NativeOptions {
        initial_window_size: Some(egui::Vec2::new(800.0, 600.0)),
        decorated: false,
        ..Default::default()
    };

    eframe::run_native(
        "Valence Packet Inspector",
        native_options,
        Box::new(move |cc| {
            let gui_app = app::GuiApp::new(cc);

            Box::new(gui_app)
        }),
    )?;

    Ok(())
}
