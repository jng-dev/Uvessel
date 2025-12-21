use anyhow::{bail, Context, Result};
use std::{
    io::Read,
    path::{Component, Path},
};

const EMBEDDED_PAYLOAD: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/app_payload.zip"));

pub fn install_payload(dest_root: &Path) -> Result<()> {
    if EMBEDDED_PAYLOAD.is_empty() {
        bail!("embedded payload is empty");
    }
    extract_zip_to(dest_root)
}

fn extract_zip_to(dest_root: &Path) -> Result<()> {
    let reader = std::io::Cursor::new(EMBEDDED_PAYLOAD);
    let mut zip = zip::ZipArchive::new(reader).context("read embedded zip")?;
    for i in 0..zip.len() {
        let mut entry = zip.by_index(i)?;
        let name = entry.name();
        let path = Path::new(name);
        if path.is_absolute()
            || path
                .components()
                .any(|c| matches!(c, Component::ParentDir | Component::Prefix(_)))
        {
            bail!("invalid path in payload zip: {name}");
        }

        let out_path = dest_root.join(path);
        if entry.is_dir() {
            std::fs::create_dir_all(&out_path)
                .with_context(|| format!("create {}", out_path.display()))?;
            continue;
        }

        if let Some(parent) = out_path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("create {}", parent.display()))?;
        }

        let mut out_file = std::fs::File::create(&out_path)
            .with_context(|| format!("create {}", out_path.display()))?;
        let mut buf = Vec::new();
        entry.read_to_end(&mut buf)?;
        std::io::Write::write_all(&mut out_file, &buf)
            .with_context(|| format!("write {}", out_path.display()))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn install_payload_extracts_embedded_zip() {
        let tmp = tempfile::tempdir().unwrap();
        install_payload(tmp.path()).unwrap();

        let placeholder = tmp
            .path()
            .join("app")
            .join("put-project-here.txt");
        assert!(placeholder.exists());
    }
}
