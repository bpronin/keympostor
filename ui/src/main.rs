#![cfg_attr(not(feature = "console"), windows_subsystem = "windows")] /* hides the console window */
use crate::ui::App;
use log::LevelFilter;
use native_windows_gui::NativeUi;
use std::fs::File;

pub mod indicator;
mod kb_watch;
mod layout;
mod profile;
mod res;
mod settings;
mod ui;
mod util;
mod win_watch;

fn main() {
    log_panics::init();
    setup_logger().expect("Failed to initialize logger.");

    App::build_ui(Default::default())
        .expect("Failed to build application UI.")
        .run();
}

fn setup_logger() -> Result<(), fern::InitError> {
    let base_config = fern::Dispatch::new()
        .format(move |out, message, record| {
            out.finish(format_args!(
                "{} {:<5} {:<32} [{}] {}",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f"),
                record.level(),
                record.target(),
                std::thread::current().name().unwrap_or("unknown"),
                message
            ))
        })
        .filter(|metadata| {
            metadata.target().starts_with("keympostor::")
        });

    let stdout_config = fern::Dispatch::new()
        .level(LevelFilter::Debug)
        .chain(std::io::stdout());

    let file_config = fern::Dispatch::new()
        .level(LevelFilter::Warn)
        .chain(File::create("keympostor.log")?);

    base_config
        .chain(stdout_config)
        .chain(file_config)
        .apply()?;

    Ok(())
}