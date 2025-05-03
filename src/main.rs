#![cfg_attr(not(feature = "dev"), windows_subsystem = "windows")]

use keympostor::ui;
use log::LevelFilter;
use simple_logger::SimpleLogger;
use std::error;

fn main() -> Result<(), Box<dyn error::Error>> {
    SimpleLogger::new().with_level(LevelFilter::Debug).init()?;
    ui::run_app();
    
    Ok(())
}
