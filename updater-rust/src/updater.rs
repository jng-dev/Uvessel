use anyhow::{bail, Context, Result};
use semver::Version;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::{
    fs::File,
    io::{self, Read},
    path::{Path, PathBuf},
    process::{Child, Command},
};

use crate::config;
use crate::ui_payload;

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
    if !should_update(config::VERSION, &manifest.version) {
        return Ok(UpdateOutcome::NoUpdate);
    }

    let app_name = app_display_name();
    let icon_path = resolve_icon_path()?;
    let mut ui_guard = launch_update_ui(&app_name, icon_path.as_deref())?;

    let installer_path = download_installer(&manifest)?;
    ui_guard.close();
    run_installer(&installer_path)?;
    Ok(UpdateOutcome::InstallerLaunched)
}

fn should_update(current: &str, incoming: &str) -> bool {
    if current.trim() == incoming.trim() {
        return false;
    }
    let current = Version::parse(current.trim());
    let incoming = Version::parse(incoming.trim());
    match (current, incoming) {
        (Ok(current), Ok(incoming)) => incoming > current,
        _ => {
            eprintln!("warning: cannot compare versions; skipping update");
            false
        }
    }
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
        .arg("--from-update")
        .spawn()
        .with_context(|| format!("launch installer {}", path.display()))?;
    Ok(())
}

fn app_display_name() -> String {
    let product = config::PRODUCT_NAME.trim();
    if !product.is_empty() {
        return product.to_string();
    }
    let name = config::NAME.trim();
    if !name.is_empty() {
        return name.to_string();
    }
    "UvesselApp".to_string()
}

fn resolve_icon_path() -> Result<Option<PathBuf>> {
    let icon = config::ICON.trim();
    if icon.is_empty() {
        return Ok(None);
    }
    let icon_path = Path::new(icon);
    if icon_path.is_absolute() {
        return Ok(icon_path.exists().then(|| icon_path.to_path_buf()));
    }
    let exe = std::env::current_exe().context("current_exe")?;
    let root = exe.parent().context("exe has no parent")?;
    let candidate = root.join(icon_path);
    Ok(candidate.exists().then_some(candidate))
}

struct UiGuard {
    child: Option<Child>,
    path: Option<PathBuf>,
}

impl UiGuard {
    fn close(&mut self) {
        if let Some(child) = self.child.as_mut() {
            let _ = child.kill();
            let _ = child.wait();
        }
        if let Some(path) = self.path.as_ref() {
            let _ = std::fs::remove_file(path);
        }
        self.child = None;
        self.path = None;
    }
}

impl Drop for UiGuard {
    fn drop(&mut self) {
        self.close();
    }
}

fn launch_update_ui(app_name: &str, icon_path: Option<&Path>) -> Result<UiGuard> {
    if ui_payload::EMBEDDED_UPDATER_UI.is_empty() {
        return Ok(UiGuard {
            child: None,
            path: None,
        });
    }

    let ui_path = write_update_ui_exe()?;
    let mut cmd = Command::new(&ui_path);
    cmd.arg("--name").arg(app_name);
    if let Some(icon_path) = icon_path {
        cmd.arg("--icon").arg(icon_path);
    }
    cmd.stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }

    let child = cmd.spawn().context("spawn updater ui")?;
    Ok(UiGuard {
        child: Some(child),
        path: Some(ui_path),
    })
}

fn write_update_ui_exe() -> Result<PathBuf> {
    let file = tempfile::Builder::new()
        .prefix("uvessel-updater-ui-")
        .suffix(".exe")
        .tempfile()
        .context("create temp updater ui")?;
    let (_, path) = file.keep().context("persist temp updater ui")?;
    std::fs::write(&path, ui_payload::EMBEDDED_UPDATER_UI)
        .with_context(|| format!("write {}", path.display()))?;
    Ok(path)
}

