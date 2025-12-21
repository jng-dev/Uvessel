use anyhow::{Context, Result};
use std::fs;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::process::Command;

fn uv_paths(root: &Path) -> (PathBuf, PathBuf, PathBuf) {
    (
        root.join("uv.exe"),
        root.join("uvx.exe"),
        root.join("uvw.exe"),
    )
}

pub fn ensure_uv(root: &Path) -> Result<()> {
    let (uv, uvx, uvw) = uv_paths(root);

    if uv.exists() {
        return Ok(());
    }

    download_and_install_uv(root)
        .context("failed to download/install uv")?;

    if !uv.exists() {
        anyhow::bail!("uv.exe not found after install at {}", uv.display());
    }

    if !uvx.exists() {
        eprintln!("warn: uvx.exe not found");
    }
    if !uvw.exists() {
        eprintln!("warn: uvw.exe not found");
    }

    // Sanity check
    let status = Command::new(&uv)
        .arg("--version")
        .status()
        .context("failed to run uv --version")?;

    if !status.success() {
        anyhow::bail!("uv --version failed");
    }

    Ok(())
}

fn download_and_install_uv(root: &Path) -> Result<()> {
    let arch = std::env::consts::ARCH;

    let asset = match arch {
        "x86_64" => "uv-x86_64-pc-windows-msvc.zip",
        "aarch64" => "uv-aarch64-pc-windows-msvc.zip",
        other => anyhow::bail!("unsupported Windows arch: {other}"),
    };

    let url = format!(
        "https://github.com/astral-sh/uv/releases/latest/download/{asset}"
    );

    let tmp_dir = tempfile::tempdir()
        .context("create temp dir")?;
    let zip_path = tmp_dir.path().join("uv.zip");

    download_file(&url, &zip_path)
        .context("download uv zip")?;

    extract_and_install(&zip_path, root)
        .context("extract/install uv")?;

    Ok(())
}

fn download_file(url: &str, dest: &Path) -> Result<()> {
    let resp = reqwest::blocking::get(url)
        .context("http GET failed")?
        .error_for_status()
        .context("http error")?;

    let mut file = fs::File::create(dest)
        .context("create zip file")?;

    let body = resp;
    io::copy(&mut body.take(u64::MAX), &mut file)
        .context("write zip file")?;

    Ok(())
}

fn extract_and_install(zip_path: &Path, root: &Path) -> Result<()> {
    let file = fs::File::open(zip_path)
        .context("open zip")?;
    let mut zip = zip::ZipArchive::new(file)
        .context("read zip")?;

    for i in 0..zip.len() {
        let mut entry = zip.by_index(i)?;
        let name = entry.name().to_owned();

        if !name.ends_with(".exe") {
            continue;
        }

        let file_name = Path::new(&name)
            .file_name()
            .context("bad zip entry name")?;

        let out_path = root.join(file_name);

        let mut out_file = fs::File::create(&out_path)
            .with_context(|| format!("create {}", out_path.display()))?;

        io::copy(&mut entry, &mut out_file)
            .with_context(|| format!("write {}", out_path.display()))?;
    }

    Ok(())
}
