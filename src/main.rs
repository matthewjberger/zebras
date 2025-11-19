mod app;

use app::Zebras;

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1400.0, 900.0])
            .with_title("Zebras - ZPL Simulator"),
        ..Default::default()
    };

    eframe::run_native(
        "Zebras",
        options,
        Box::new(|_cc| Ok(Box::new(Zebras::default()))),
    )
}
