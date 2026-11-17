//! Braupause Sud-Editor: Desktop-Werkzeug zum Anlegen und Pflegen der
//! Sud-Dateien in `batches/` inklusive Validierung, Commit und Push.

mod app;
mod datetime;
mod git;
mod images;
mod model;
mod tabs;
mod theme;
mod validate;
mod vocab;
mod widgets;

fn main() -> eframe::Result {
    let cli_path = std::env::args().nth(1).map(std::path::PathBuf::from);
    let mut viewport = egui::ViewportBuilder::default();
    if let Ok(icon) = eframe::icon_data::from_png_bytes(include_bytes!("../assets/editor-icon.png")) {
        viewport = viewport.with_icon(icon);
    }
    let options = eframe::NativeOptions {
        viewport: viewport
            .with_title("Braupause – Sud-Editor")
            .with_inner_size([1440.0, 900.0])
            .with_min_inner_size([980.0, 600.0]),
        ..Default::default()
    };
    eframe::run_native(
        "braupause-editor",
        options,
        Box::new(move |cc| Ok(Box::new(app::BrauApp::new(cc, cli_path)))),
    )
}
