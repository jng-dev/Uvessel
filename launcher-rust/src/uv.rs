use anyhow::{bail, Context, Result};
use std::{
    fs,
    io::{self, Read},
    path::{Path, PathBuf},
};

const UV_LATEST_BASE: &str = "https://github.com/astral-sh/uv/releases/latest/download";

pub fn ensure_uv(root: &Path) -> Result<()> {
    let (uv, _, _) = uv_paths(root);
    if uv.exists() {
        return Ok(());
    }

    let arch = std::env::consts::ARCH;
    let asset = asset_name_for_arch(arch)?;
    let url = format!("{UV_LATEST_BASE}/{asset}");

    let tmp_dir = tempfile::tempdir().context("create temp dir")?;
    let zip_path = tmp_dir.path().join("uv.zip");
    download_file(&url, &zip_path).context("download uv zip")?;
    install_from_zip(root, &zip_path).context("install uv from zip")?;

    if !uv.exists() {
        bail!("uv.exe not found after install at {}", uv.display());
    }

    Ok(())
}

pub fn install_from_zip(root: &Path, zip_path: &Path) -> Result<()> {
    let file = fs::File::open(zip_path).context("open zip")?;
    let mut zip = zip::ZipArchive::new(file).context("read zip")?;

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

fn asset_name_for_arch(arch: &str) -> Result<&'static str> {
    match arch {
        "x86_64" => Ok("uv-x86_64-pc-windows-msvc.zip"),
        "aarch64" => Ok("uv-aarch64-pc-windows-msvc.zip"),
        other => bail!("unsupported Windows arch: {other}"),
    }
}

fn uv_paths(root: &Path) -> (PathBuf, PathBuf, PathBuf) {
    (
        root.join("uv.exe"),
        root.join("uvx.exe"),
        root.join("uvw.exe"),
    )
}

fn download_file(url: &str, dest: &Path) -> Result<()> {
    let resp = reqwest::blocking::get(url)
        .context("http GET failed")?
        .error_for_status()
        .context("http error")?;

    let mut file = fs::File::create(dest).context("create zip file")?;
    let body = resp;
    io::copy(&mut body.take(u64::MAX), &mut file).context("write zip file")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn asset_name_for_arch_maps_known_arches() {
        assert_eq!(
            asset_name_for_arch("x86_64").unwrap(),
            "uv-x86_64-pc-windows-msvc.zip"
        );
        assert_eq!(
            asset_name_for_arch("aarch64").unwrap(),
            "uv-aarch64-pc-windows-msvc.zip"
        );
    }

    #[test]
    fn asset_name_for_arch_rejects_unknown() {
        let err = asset_name_for_arch("mips").unwrap_err();
        assert!(err.to_string().contains("unsupported Windows arch"));
    }

    #[test]
    fn install_from_zip_extracts_exes() {
        let tmp = tempfile::tempdir().unwrap();
        let zip_path = tmp.path().join("uv.zip");
        create_uv_zip(&zip_path).unwrap();

        let dest = tmp.path().join("root");
        fs::create_dir_all(&dest).unwrap();
        install_from_zip(&dest, &zip_path).unwrap();

        let (uv, uvx, uvw) = uv_paths(&dest);
        assert!(uv.exists());
        assert!(uvx.exists());
        assert!(uvw.exists());
    }

    fn create_uv_zip(path: &Path) -> Result<()> {
        let file = fs::File::create(path).context("create zip")?;
        let mut zip = zip::ZipWriter::new(file);
        let options = zip::write::FileOptions::default();

        for name in ["uv.exe", "uvx.exe", "uvw.exe"] {
            zip.start_file(name, options).context("start file")?;
            zip.write_all(b"dummy").context("write entry")?;
        }
        zip.finish().context("finish zip")?;
        Ok(())
    }
}
