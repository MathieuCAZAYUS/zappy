mod command;
mod config;
mod constants;
mod entities;
mod game;
mod gui;
mod server;
mod world;

use crate::constants::{ERROR_EXIT, USAGE};
use std::process::ExitCode;

fn main() -> ExitCode {
    match server::runtime::run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            eprintln!("{USAGE}");
            ExitCode::from(ERROR_EXIT as u8)
        }
    }
}
