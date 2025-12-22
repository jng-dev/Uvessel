use std::{
    fs::{self, File},
    io::{self, Read, Write},
    path::{Path, PathBuf},
};
use serde::Deserialize;

fn main() {
    let out_dir = std::env::var("OUT_DIR").expect("OUT_DIR not set");
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    let manifest_dir = PathBuf::from(manifest_dir);
    let repo_root = manifest_dir.join("..");
    let app_dir = repo_root.join("app");
    println!("cargo:rerun-if-changed={}", app_dir.display());
    let shim_path = manifest_dir.join("embedded").join("launcher.exe");
    let updater_path = manifest_dir.join("embedded").join("updater.exe");
    println!("cargo:rerun-if-changed={}", shim_path.display());
    println!("cargo:rerun-if-changed={}", updater_path.display());
    let out_path = PathBuf::from(&out_dir).join("app_payload.zip");
    let root_dir = app_dir.parent().unwrap_or(&app_dir);
    let config = load_config(&repo_root).unwrap_or_else(|err| {
        panic!("failed to load config.toml: {err}");
    });

    if !shim_path.exists() {
        panic!(
            "embedded shim not found at {} (build the shim first)",
            shim_path.display()
        );
    }
    if !updater_path.exists() {
        panic!(
            "embedded updater not found at {} (build the updater first)",
            updater_path.display()
        );
    }

    if !app_dir.exists() {
        panic!("app/ directory not found; cannot embed payload");
    }

    if let Err(err) = write_payload_zip(&app_dir, root_dir, &out_path) {
        panic!("failed to build payload zip: {err}");
    }

    if let Err(err) = embed_icon(&repo_root, &config) {
        panic!("failed to embed icon: {err}");
    }

    if let Err(err) = write_config_rs(&PathBuf::from(&out_dir), &config) {
        panic!("failed to write config: {err}");
    }
}

fn write_payload_zip(app_dir: &Path, root_dir: &Path, out_path: &Path) -> io::Result<()> {
    let file = File::create(out_path)?;
    let mut zip = zip::ZipWriter::new(file);
    let options = zip::write::FileOptions::default();

    add_dir_recursive("app", app_dir, app_dir, &mut zip, options)?;
    add_optional_dir("assets", root_dir, &mut zip, options)?;
    add_optional_dir("data", root_dir, &mut zip, options)?;

    zip.finish()?;
    Ok(())
}

fn add_dir_recursive(
    prefix: &str,
    root: &Path,
    dir: &Path,
    zip: &mut zip::ZipWriter<File>,
    options: zip::write::FileOptions,
) -> io::Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            add_dir_recursive(prefix, root, &path, zip, options)?;
        } else if path.is_file() {
            let rel = path.strip_prefix(root).unwrap_or(&path);
            let name = Path::new(prefix).join(rel);
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

fn add_optional_dir(
    name: &str,
    root_dir: &Path,
    zip: &mut zip::ZipWriter<File>,
    options: zip::write::FileOptions,
) -> io::Result<()> {
    let dir = root_dir.join(name);
    if !dir.exists() {
        return Ok(());
    }
    add_dir_recursive(name, &dir, &dir, zip, options)
}

#[derive(Debug, Deserialize)]
struct Config {
    app_id: String,
    name: String,
    product_name: String,
    company: String,
    description: String,
    version: String,
    entry_point: String,
    #[serde(default)]
    icon: String,
    #[serde(default)]
    uvessel_instance_link: String,
    #[serde(default)]
    auto_update_enabled: bool,
    #[serde(default)]
    update_manifest_url: String,
    #[serde(default)]
    install_dir: String,
}

fn load_config(repo_root: &Path) -> io::Result<Config> {
    let config_path = repo_root.join("config.toml");
    println!("cargo:rerun-if-changed={}", config_path.display());
    let contents = fs::read_to_string(&config_path)?;
    let cfg: Config = toml::from_str(&contents)
        .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?;
    Ok(cfg)
}

fn embed_icon(repo_root: &Path, config: &Config) -> io::Result<()> {
    let icon_path = resolve_icon_path(repo_root, config);
    let mut res = winres::WindowsResource::new();
    if let Some(icon_path) = icon_path {
        res.set_icon(icon_path.to_string_lossy().as_ref());
    }
    if !config.product_name.is_empty() {
        res.set("ProductName", &config.product_name);
    }
    if !config.description.is_empty() {
        res.set("FileDescription", &config.description);
    }
    if !config.company.is_empty() {
        res.set("CompanyName", &config.company);
    }
    if !config.version.is_empty() {
        res.set("FileVersion", &config.version);
        res.set("ProductVersion", &config.version);
    }
    if !config.app_id.is_empty() {
        res.set("InternalName", &config.app_id);
    }
    res.compile()?;
    Ok(())
}

fn resolve_icon_path(repo_root: &Path, config: &Config) -> Option<PathBuf> {
    if !config.icon.is_empty() {
        let candidate = repo_root.join(&config.icon);
        if candidate.exists() {
            return Some(candidate);
        }
    }
    let assets_dir = repo_root.join("assets");
    if !assets_dir.exists() {
        return None;
    }
    let mut ico_paths: Vec<PathBuf> = fs::read_dir(&assets_dir).ok()?
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| path.extension().map(|e| e.eq_ignore_ascii_case("ico")).unwrap_or(false))
        .collect();
    ico_paths.sort();
    ico_paths.first().cloned()
}

fn write_config_rs(out_dir: &Path, config: &Config) -> io::Result<()> {
    let out_path = out_dir.join("uvessel_config.rs");
    let mut file = File::create(&out_path)?;
    writeln!(file, "pub const APP_ID: &str = {:?};", config.app_id)?;
    writeln!(file, "pub const NAME: &str = {:?};", config.name)?;
    writeln!(file, "pub const PRODUCT_NAME: &str = {:?};", config.product_name)?;
    writeln!(file, "pub const COMPANY: &str = {:?};", config.company)?;
    writeln!(file, "pub const DESCRIPTION: &str = {:?};", config.description)?;
    writeln!(file, "pub const VERSION: &str = {:?};", config.version)?;
    writeln!(file, "pub const ENTRY_POINT: &str = {:?};", config.entry_point)?;
    writeln!(file, "pub const ICON: &str = {:?};", config.icon)?;
    writeln!(
        file,
        "pub const UVESSEL_INSTANCE_LINK: &str = {:?};",
        config.uvessel_instance_link
    )?;
    writeln!(
        file,
        "pub const AUTO_UPDATE_ENABLED: bool = {:?};",
        config.auto_update_enabled
    )?;
    writeln!(
        file,
        "pub const UPDATE_MANIFEST_URL: &str = {:?};",
        config.update_manifest_url
    )?;
    writeln!(
        file,
        "pub const INSTALL_DIR: &str = {:?};",
        config.install_dir
    )?;
    Ok(())
}
