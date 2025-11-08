use keympostor::win_watch::detect_window_activation;
use log::{debug, LevelFilter};
use regex::Regex;
use simple_logger::SimpleLogger;
use std::str::FromStr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::thread::JoinHandle;

fn setup_logger() {
    SimpleLogger::new()
        .with_level(LevelFilter::Debug)
        .init()
        .expect("Failed to initialize logger.");
}

fn setup_exit_handler(run_handler: &Arc<AtomicBool>) {
    let rh = Arc::clone(run_handler);
    ctrlc::set_handler(move || {
        println!("Shutting down...");
        rh.store(false, Ordering::SeqCst)
    })
    .unwrap();
}

fn start_window_detection(run_handle: &Arc<AtomicBool>) -> JoinHandle<()> {
    let rh = Arc::clone(run_handle);
    thread::spawn(move || {
        let rules = vec![
            Regex::from_str("Chrome").unwrap(),
            Regex::from_str("TOTALCMD64.EXE").unwrap(),
        ];
        detect_window_activation(rules, Box::new(on_window_activate), rh);
    })
}

fn on_window_activate(rule: Option<&Regex>) {
    match rule {
        Some(rule) => {
            debug!("Watch windows activated on rule: '{}'", rule);
        }
        None => {
            debug!("No active watch windows");
        }
    }
}

fn main() {
    setup_logger();

    //TODO: replace multithreading with tokio
    let run_handle = Arc::new(AtomicBool::new(true));
    setup_exit_handler(&run_handle);

    println!("Running... (press Ctrl+C to stop)");

    let jobs = vec![start_window_detection(&run_handle)];

    for task in jobs {
        task.join().unwrap();
    }
}
