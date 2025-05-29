#![cfg_attr(not(feature = "console"), windows_subsystem = "windows")]

use log::LevelFilter;
use simple_logger::SimpleLogger;

mod res;
mod settings;
mod ui;

fn main() {
    SimpleLogger::new()
        .with_level(LevelFilter::Debug)
        .init()
        .expect("Failed to initialize logger.");

    ui::run_app();
}
