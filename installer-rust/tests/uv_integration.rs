#[path = "../src/uv.rs"]
mod uv;

use std::{fs, io::Write, path::Path};

#[test]
fn install_from_zip_writes_uv_binaries() {
    let tmp = tempfile::tempdir().unwrap();
    let zip_path = tmp.path().join("uv.zip");
    create_uv_zip(&zip_path).unwrap();

    let root = tmp.path().join("root");
    fs::create_dir_all(&root).unwrap();

    uv::install_from_zip(&root, &zip_path).unwrap();

    assert!(root.join("uv.exe").exists());
    assert!(root.join("uvx.exe").exists());
    assert!(root.join("uvw.exe").exists());
}

fn create_uv_zip(path: &Path) -> anyhow::Result<()> {
    let file = fs::File::create(path)?;
    let mut zip = zip::ZipWriter::new(file);
    let options = zip::write::FileOptions::default();
    for name in ["uv.exe", "uvx.exe", "uvw.exe"] {
        zip.start_file(name, options)?;
        zip.write_all(b"dummy")?;
    }
    zip.finish()?;
    Ok(())
}
