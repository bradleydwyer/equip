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
