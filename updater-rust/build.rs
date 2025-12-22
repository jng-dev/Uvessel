use serde::Deserialize;
use std::{
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
};

#[derive(Debug, Deserialize)]
struct Config {
    version: String,
    #[serde(default)]
    uvessel_instance_link: String,
    #[serde(default)]
    auto_update_enabled: bool,
    #[serde(default)]
    update_manifest_url: String,
}

fn main() {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    let manifest_dir = PathBuf::from(manifest_dir);
    let repo_root = manifest_dir.join("..");
    let config = load_config(&repo_root).unwrap_or_else(|err| {
        panic!("failed to load config.toml: {err}");
    });

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

fn write_config_rs(out_dir: &Path, config: &Config) -> io::Result<()> {
    let out_path = out_dir.join("uvessel_config.rs");
    let mut file = fs::File::create(&out_path)?;
    writeln!(file, "pub const VERSION: &str = {:?};", config.version)?;
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
    Ok(())
}
