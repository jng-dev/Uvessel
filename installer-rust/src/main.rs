#![cfg_attr(windows, windows_subsystem = "windows")]

mod fs_ops;
mod installer;
mod paths;
mod payload;
mod shortcuts;
mod shim_payload;
mod state;
mod uv;
mod config;
mod ui_payload;

use anyhow::Result;

fn main() -> Result<()> {
    let root = paths::root_dir()?;
    installer::run(&root)
}
