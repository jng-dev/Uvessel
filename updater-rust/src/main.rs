mod config;
mod updater;

use anyhow::Result;

fn main() -> Result<()> {
    let outcome = updater::run();
    match outcome {
        updater::UpdateOutcome::InstallerLaunched => std::process::exit(10),
        updater::UpdateOutcome::NoUpdate => Ok(()),
    }
}
