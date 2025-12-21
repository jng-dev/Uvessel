use anyhow::{bail, Context, Result};
use serde::Deserialize;
use std::{
    env,
    fs,
    path::{Path, PathBuf},
    process::Command,
};

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
}

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    let repo_root = env::current_dir().context("current_dir")?;
    let config_path = parse_arg(&args, "--config")
        .map(PathBuf::from)
        .unwrap_or_else(|| repo_root.join("config.toml"));
    let out_dir = parse_arg(&args, "--out-dir")
        .map(PathBuf::from)
        .unwrap_or_else(|| repo_root.join("dist"));

    let config = load_config(&config_path)?;
    validate_config(&config, &repo_root)?;

    let launcher_dir = repo_root.join("launcher-rust");
    build_launcher(&launcher_dir)?;

    let exe_name = sanitize_exe_name(&config.product_name);
    let built_exe = launcher_dir
        .join("target")
        .join("release")
        .join("launcher.exe");
    if !built_exe.exists() {
        bail!("launcher.exe not found at {}", built_exe.display());
    }

    fs::create_dir_all(&out_dir).context("create output dir")?;
    let dest_exe = out_dir.join(format!("{exe_name}.exe"));
    fs::copy(&built_exe, &dest_exe).with_context(|| {
        format!(
            "copy {} -> {}",
            built_exe.display(),
            dest_exe.display()
        )
    })?;

    println!("built {}", dest_exe.display());
    Ok(())
}

fn parse_arg(args: &[String], name: &str) -> Option<String> {
    args.iter()
        .position(|arg| arg == name)
        .and_then(|idx| args.get(idx + 1))
        .cloned()
}

fn load_config(config_path: &Path) -> Result<Config> {
    let contents = fs::read_to_string(config_path)
        .with_context(|| format!("read {}", config_path.display()))?;
    let cfg: Config = toml::from_str(&contents).context("parse config.toml")?;
    Ok(cfg)
}

fn validate_config(config: &Config, repo_root: &Path) -> Result<()> {
    require_field("app_id", &config.app_id)?;
    require_field("name", &config.name)?;
    require_field("product_name", &config.product_name)?;
    require_field("company", &config.company)?;
    require_field("description", &config.description)?;
    require_field("version", &config.version)?;
    require_field("entry_point", &config.entry_point)?;

    if !config.icon.is_empty() {
        let icon_path = repo_root.join(&config.icon);
        if !icon_path.exists() {
            bail!("icon not found at {}", icon_path.display());
        }
    }

    Ok(())
}

fn require_field(name: &str, value: &str) -> Result<()> {
    if value.trim().is_empty() {
        bail!("config field {name} is required");
    }
    Ok(())
}

fn build_launcher(launcher_dir: &Path) -> Result<()> {
    let status = Command::new("cargo")
        .arg("build")
        .arg("--release")
        .current_dir(launcher_dir)
        .status()
        .with_context(|| format!("build in {}", launcher_dir.display()))?;
    if !status.success() {
        bail!("cargo build failed (exit {:?})", status.code());
    }
    Ok(())
}

fn sanitize_exe_name(name: &str) -> String {
    let trimmed = name.trim();
    let mut out = String::new();
    for ch in trimmed.chars() {
        if ch.is_ascii_alphanumeric() || ch == ' ' || ch == '-' || ch == '_' || ch == '.' {
            out.push(ch);
        } else {
            out.push('_');
        }
    }
    let out = out.trim().trim_end_matches('.').trim_end_matches(' ').to_string();
    if out.is_empty() {
        "UvesselApp".to_string()
    } else {
        out
    }
}
