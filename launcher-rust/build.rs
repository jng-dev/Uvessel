use serde::Deserialize;
use std::{
    fs,
    io,
    path::{Path, PathBuf},
};

#[derive(Debug, Deserialize)]
struct Config {
    app_id: String,
    name: String,
    product_name: String,
    company: String,
    description: String,
    version: String,
    #[serde(default)]
    icon: String,
}

fn main() {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    let manifest_dir = PathBuf::from(manifest_dir);
    let repo_root = manifest_dir.join("..");
    let config = load_config(&repo_root).unwrap_or_else(|err| {
        panic!("failed to load config.toml: {err}");
    });

    if let Err(err) = embed_icon(&repo_root, &config) {
        panic!("failed to embed icon: {err}");
    }

    if let Err(err) = write_config_rs(&PathBuf::from(std::env::var("OUT_DIR").unwrap()), &config) {
        panic!("failed to write config: {err}");
    }
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
    use std::io::Write;
    let out_path = out_dir.join("uvessel_config.rs");
    let mut file = fs::File::create(&out_path)?;
    writeln!(file, "pub const APP_ID: &str = {:?};", config.app_id)?;
    writeln!(file, "pub const NAME: &str = {:?};", config.name)?;
    writeln!(file, "pub const PRODUCT_NAME: &str = {:?};", config.product_name)?;
    Ok(())
}
