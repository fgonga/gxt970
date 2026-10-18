mod app;
mod keys;
mod settings;
mod theme;
mod worker;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("GXT 970")
            .with_inner_size([1030.0, 662.0])
            .with_min_inner_size([900.0, 600.0]),
        ..Default::default()
    };
    eframe::run_native("GXT 970", options, Box::new(|cc| Ok(Box::new(app::App::new(cc)))))
}
