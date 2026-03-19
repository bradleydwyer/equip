// Utility module — not all colors used yet
#![allow(dead_code)]

const RESET: &str = "\x1b[0m";
const BOLD: &str = "\x1b[1m";
const DIM: &str = "\x1b[2m";
const RED: &str = "\x1b[31m";
const GREEN: &str = "\x1b[32m";
const YELLOW: &str = "\x1b[33m";
const CYAN: &str = "\x1b[36m";

pub fn green(s: &str) -> String {
    format!("{GREEN}{s}{RESET}")
}

pub fn red(s: &str) -> String {
    format!("{RED}{s}{RESET}")
}

pub fn yellow(s: &str) -> String {
    format!("{YELLOW}{s}{RESET}")
}

pub fn cyan(s: &str) -> String {
    format!("{CYAN}{s}{RESET}")
}

pub fn dim(s: &str) -> String {
    format!("{DIM}{s}{RESET}")
}

pub fn bold(s: &str) -> String {
    format!("{BOLD}{s}{RESET}")
}

use std::io::Write;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

/// A braille dot spinner that runs in a background thread.
pub struct Spinner {
    running: Arc<AtomicBool>,
    handle: Option<std::thread::JoinHandle<()>>,
}

const FRAMES: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

impl Spinner {
    /// Start a spinner with a label. Returns a handle to stop it.
    pub fn start(label: &str) -> Self {
        let running = Arc::new(AtomicBool::new(true));
        let running_clone = running.clone();
        let label = label.to_string();

        let handle = std::thread::spawn(move || {
            let mut i = 0;
            while running_clone.load(Ordering::Relaxed) {
                let frame = FRAMES[i % FRAMES.len()];
                print!("\r  {CYAN}{frame}{RESET} {BOLD}{label}{RESET}");
                let _ = std::io::stdout().flush();
                i += 1;
                std::thread::sleep(std::time::Duration::from_millis(80));
            }
        });

        Spinner {
            running,
            handle: Some(handle),
        }
    }

    /// Stop the spinner and clear the line.
    pub fn stop(self) {
        self.running.store(false, Ordering::Relaxed);
        if let Some(h) = self.handle {
            let _ = h.join();
        }
        // Clear the spinner line
        print!("\r\x1b[2K");
        let _ = std::io::stdout().flush();
    }
}
