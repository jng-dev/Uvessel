use anyhow::{bail, Context, Result};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::{
    fs::File,
    io::{self, Read},
    path::{Path, PathBuf},
    process::Command,
};

use crate::config;

#[derive(Debug, Deserialize)]
struct Manifest {
    version: String,
    installer_url: String,
    #[serde(default)]
    sha256: String,
}

pub enum UpdateOutcome {
    NoUpdate,
    InstallerLaunched,
}

pub fn run() -> UpdateOutcome {
    match run_inner() {
        Ok(outcome) => outcome,
        Err(err) => {
            eprintln!("warning: updater failed: {err}");
            UpdateOutcome::NoUpdate
        }
    }
}

fn run_inner() -> Result<UpdateOutcome> {
    if !config::AUTO_UPDATE_ENABLED {
        return Ok(UpdateOutcome::NoUpdate);
    }

    let manifest_url = resolve_manifest_url();
    if manifest_url.is_empty() {
        return Ok(UpdateOutcome::NoUpdate);
    }

    let manifest = fetch_manifest(&manifest_url)?;
    if manifest.version.trim() == config::VERSION.trim() {
        return Ok(UpdateOutcome::NoUpdate);
    }

    let installer_path = download_installer(&manifest)?;
    run_installer(&installer_path)?;
    Ok(UpdateOutcome::InstallerLaunched)
}

fn resolve_manifest_url() -> String {
    let configured = config::UPDATE_MANIFEST_URL.trim();
    if !configured.is_empty() {
        return configured.to_string();
    }
    let base = config::UVESSEL_INSTANCE_LINK.trim();
    if base.is_empty() {
        return String::new();
    }
    let base = base.trim_end_matches('/');
    format!("{base}/latest/download/latest.json")
}

fn fetch_manifest(url: &str) -> Result<Manifest> {
    let resp = reqwest::blocking::get(url)
        .with_context(|| format!("fetch manifest {url}"))?
        .error_for_status()
        .with_context(|| format!("manifest request failed for {url}"))?;
    let manifest = resp.json::<Manifest>().context("parse manifest JSON")?;
    if manifest.version.trim().is_empty() {
        bail!("manifest version is empty");
    }
    if manifest.installer_url.trim().is_empty() {
        bail!("manifest installer_url is empty");
    }
    Ok(manifest)
}

fn download_installer(manifest: &Manifest) -> Result<PathBuf> {
    let mut resp = reqwest::blocking::get(&manifest.installer_url)
        .with_context(|| format!("download installer {}", manifest.installer_url))?
        .error_for_status()
        .context("installer request failed")?;
    let mut file = tempfile::Builder::new()
        .prefix("uvessel-installer-")
        .suffix(".exe")
        .tempfile()
        .context("create temp installer file")?;
    io::copy(&mut resp, &mut file).context("write installer to temp file")?;
    let (_, path) = file.keep().context("persist temp installer file")?;

    if !manifest.sha256.trim().is_empty() {
        let actual = sha256_file(&path)?;
        let expected = normalize_hex(&manifest.sha256);
        if actual != expected {
            bail!("installer sha256 mismatch");
        }
    }

    Ok(path)
}

fn sha256_file(path: &Path) -> Result<String> {
    let mut file = File::open(path).with_context(|| format!("open {}", path.display()))?;
    let mut hasher = Sha256::new();
    let mut buf = [0u8; 8192];
    loop {
        let read = file.read(&mut buf)?;
        if read == 0 {
            break;
        }
        hasher.update(&buf[..read]);
    }
    Ok(hex::encode(hasher.finalize()))
}

fn normalize_hex(value: &str) -> String {
    value
        .trim()
        .trim_start_matches("sha256:")
        .to_ascii_lowercase()
}

fn run_installer(path: &Path) -> Result<()> {
    Command::new(path)
        .spawn()
        .with_context(|| format!("launch installer {}", path.display()))?;
    Ok(())
}

