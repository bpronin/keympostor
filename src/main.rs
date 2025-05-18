#![cfg_attr(not(feature = "dev"), windows_subsystem = "windows")]

use keympostor::ui;
use log::LevelFilter;
use simple_logger::SimpleLogger;

fn main() {
    SimpleLogger::new()
        .with_level(LevelFilter::Debug)
        .init()
        .expect("Failed to initialize logger.");

    ui::run_app();
}
