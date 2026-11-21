#![cfg_attr(windows, windows_subsystem = "windows")]

//! Entry point for the native, offline Music Data desktop application.

mod metadata;
#[cfg(windows)]
mod windows;

fn main() {
    #[cfg(windows)]
    windows::run();
    #[cfg(not(windows))]
    eprintln!("Music Data Desktop supports Windows 11 x64.");
}
