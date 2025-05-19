use log::LevelFilter;
use simple_logger::SimpleLogger;

mod keyboard;
mod res;
mod settings;
mod ui;
mod util;

fn main() {
    SimpleLogger::new()
        .with_level(LevelFilter::Debug)
        .init()
        .expect("Failed to initialize logger.");

    ui::run_app();
}
