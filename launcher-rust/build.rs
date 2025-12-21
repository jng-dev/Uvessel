use std::{
    fs::{self, File},
    io::{self, Read, Write},
    path::{Path, PathBuf},
};

fn main() {
    let out_dir = std::env::var("OUT_DIR").expect("OUT_DIR not set");
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    let app_dir = PathBuf::from(manifest_dir).join("..").join("app");
    println!("cargo:rerun-if-changed={}", app_dir.display());
    let out_path = PathBuf::from(out_dir).join("app_payload.zip");

    if !app_dir.exists() {
        panic!("app/ directory not found; cannot embed payload");
    }

    if let Err(err) = write_app_zip(&app_dir, &out_path) {
        panic!("failed to build payload zip: {err}");
    }

    if let Err(err) = embed_icon(&app_dir) {
        panic!("failed to embed icon: {err}");
    }
}

fn write_app_zip(app_dir: &Path, out_path: &Path) -> io::Result<()> {
    let file = File::create(out_path)?;
    let mut zip = zip::ZipWriter::new(file);
    let options = zip::write::FileOptions::default();

    add_dir_recursive(app_dir, app_dir, &mut zip, options)?;

    zip.finish()?;
    Ok(())
}

fn add_dir_recursive(
    root: &Path,
    dir: &Path,
    zip: &mut zip::ZipWriter<File>,
    options: zip::write::FileOptions,
) -> io::Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            add_dir_recursive(root, &path, zip, options)?;
        } else if path.is_file() {
            let rel = path.strip_prefix(root).unwrap_or(&path);
            let name = Path::new("app").join(rel);
            let name = name.to_string_lossy().replace('\\', "/");
            zip.start_file(name, options)?;
            let mut f = File::open(&path)?;
            let mut buf = Vec::new();
            f.read_to_end(&mut buf)?;
            zip.write_all(&buf)?;
        }
    }
    Ok(())
}

fn embed_icon(app_dir: &Path) -> io::Result<()> {
    let media_dir = app_dir.parent().unwrap_or(app_dir).join("media");
    if !media_dir.exists() {
        return Ok(());
    }
    let mut ico_paths: Vec<PathBuf> = fs::read_dir(&media_dir)?
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| path.extension().map(|e| e.eq_ignore_ascii_case("ico")).unwrap_or(false))
        .collect();
    ico_paths.sort();
    let Some(ico_path) = ico_paths.first() else {
        return Ok(());
    };

    let mut res = winres::WindowsResource::new();
    res.set_icon(ico_path.to_string_lossy().as_ref());
    res.compile()?;
    Ok(())
}
