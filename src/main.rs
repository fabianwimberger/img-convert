mod app;
mod batch;
mod codec;
mod files;
mod settings;
mod theme;
mod ui;

use eframe::egui;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([900.0, 700.0])
            .with_min_inner_size([760.0, 620.0]),
        ..Default::default()
    };

    eframe::run_native(
        "img-convert",
        options,
        Box::new(|_cc| Ok(Box::new(app::App::default()))),
    )
}
