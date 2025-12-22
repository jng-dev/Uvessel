#![cfg_attr(windows, windows_subsystem = "windows")]

mod fs_ops;
mod installer;
mod logging;
mod paths;
mod payload;
mod shortcuts;
mod shim_payload;
mod state;
mod uv;
mod config;

use anyhow::Result;

fn main() -> Result<()> {
    let root = paths::root_dir()?;
    logging::init(&root)?;
    installer::run(&root)
}
