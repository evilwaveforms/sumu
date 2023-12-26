use std::env;
use sumu::Sumu;

fn main() {
    env::set_var("RUST_BACKTRACE", "1");
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([400.0, 300.0])
            .with_min_inner_size([300.0, 220.0]),
            ..Default::default()
    };
    let _ = eframe::run_native(
        "sumu",
        native_options,
        Box::new(|cc| Box::new(Sumu::new(cc))),
    );
}
