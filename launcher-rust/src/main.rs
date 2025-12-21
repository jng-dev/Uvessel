#![cfg_attr(windows, windows_subsystem = "windows")]

mod fs_ops;
mod installer;
mod logging;
mod paths;
mod payload;
mod runner;
mod shortcuts;
mod state;
mod uv;

use anyhow::Result;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Installer,
    Runner,
}

fn main() -> Result<()> {
    let root = paths::root_dir()?;
    logging::init(&root)?;
    match select_mode(&root) {
        Mode::Runner => runner::run(&root),
        Mode::Installer => installer::run(&root),
    }
}

pub fn select_mode(root: &Path) -> Mode {
    if state::state_exists(root) {
        Mode::Runner
    } else {
        Mode::Installer
    }
}
