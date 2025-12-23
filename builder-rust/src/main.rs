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
    #[serde(default)]
    install_dir: String,
}

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    let repo_root = find_repo_root()?;
    let config_path = parse_arg(&args, "--config")
        .map(PathBuf::from)
        .map(|p| absolutize_path(&repo_root, p))
        .unwrap_or_else(|| repo_root.join("config.toml"));
    let out_dir = parse_arg(&args, "--out-dir")
        .map(PathBuf::from)
        .map(|p| absolutize_path(&repo_root, p))
        .unwrap_or_else(|| repo_root.join("dist"));

    let config = load_config(&config_path)?;
    validate_config(&config, &repo_root)?;

    let installer_dir = repo_root.join("installer-rust");
    let shim_dir = repo_root.join("launcher-rust");
    let updater_dir = repo_root.join("updater-rust");
    let installer_ui_dir = repo_root.join("tauri-ui-rust").join("webview-installer-rust");
    let updater_ui_dir = repo_root
        .join("tauri-ui-rust")
        .join("webview-updater-rust")
        .join("src-tauri");

    build_launcher(&shim_dir)?;
    build_updater_ui(&updater_ui_dir)?;
    stage_updater_ui_for_updater(&updater_ui_dir, &updater_dir)?;
    build_updater(&updater_dir)?;
    build_installer_ui(&installer_ui_dir)?;
    stage_shim_for_installer(&shim_dir, &installer_dir)?;
    stage_updater_for_installer(&updater_dir, &installer_dir)?;
    stage_installer_ui_for_installer(&installer_ui_dir, &installer_dir)?;
    build_launcher(&installer_dir)?;

    let exe_name = format!("{}-installer", sanitize_exe_name(&config.product_name));
    let built_exe = installer_dir
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

fn build_updater(updater_dir: &Path) -> Result<()> {
    let status = Command::new("cargo")
        .arg("build")
        .arg("--release")
        .current_dir(updater_dir)
        .status()
        .with_context(|| format!("build in {}", updater_dir.display()))?;
    if !status.success() {
        bail!("cargo build failed (exit {:?})", status.code());
    }
    Ok(())
}

fn stage_shim_for_installer(shim_dir: &Path, installer_dir: &Path) -> Result<()> {
    let shim_exe = shim_dir
        .join("target")
        .join("release")
        .join("launcher.exe");
    if !shim_exe.exists() {
        bail!("shim launcher.exe not found at {}", shim_exe.display());
    }
    let embedded_dir = installer_dir.join("embedded");
    fs::create_dir_all(&embedded_dir).context("create embedded dir")?;
    let dest = embedded_dir.join("launcher.exe");
    fs::copy(&shim_exe, &dest).with_context(|| {
        format!(
            "copy {} -> {}",
            shim_exe.display(),
            dest.display()
        )
    })?;
    Ok(())
}

fn stage_updater_for_installer(updater_dir: &Path, installer_dir: &Path) -> Result<()> {
    let updater_exe = updater_dir
        .join("target")
        .join("release")
        .join("updater.exe");
    if !updater_exe.exists() {
        bail!("updater.exe not found at {}", updater_exe.display());
    }
    let embedded_dir = installer_dir.join("embedded");
    fs::create_dir_all(&embedded_dir).context("create embedded dir")?;
    let dest = embedded_dir.join("updater.exe");
    fs::copy(&updater_exe, &dest).with_context(|| {
        format!(
            "copy {} -> {}",
            updater_exe.display(),
            dest.display()
        )
    })?;
    Ok(())
}

fn build_installer_ui(ui_dir: &Path) -> Result<()> {
    if !ui_dir.exists() {
        bail!("installer ui dir not found at {}", ui_dir.display());
    }
    let node_modules = ui_dir.join("node_modules");
    if !node_modules.exists() {
        let status = run_npm(ui_dir, &["install"])?;
        if !status.success() {
            bail!("npm install failed (exit {:?})", status.code());
        }
    }

    let status = run_npm(ui_dir, &["run", "tauri", "build"])?;
    if !status.success() {
        bail!("tauri build failed (exit {:?})", status.code());
    }
    Ok(())
}

fn build_updater_ui(ui_dir: &Path) -> Result<()> {
    if !ui_dir.exists() {
        bail!("updater ui dir not found at {}", ui_dir.display());
    }
    let status = Command::new("cargo")
        .arg("build")
        .arg("--release")
        .current_dir(ui_dir)
        .status()
        .with_context(|| format!("build updater ui in {}", ui_dir.display()))?;
    if !status.success() {
        bail!("updater ui build failed (exit {:?})", status.code());
    }
    Ok(())
}

fn run_npm(ui_dir: &Path, args: &[&str]) -> Result<std::process::ExitStatus> {
    let mut cmd = if cfg!(windows) {
        let mut cmd = Command::new("cmd");
        cmd.arg("/C").arg("npm");
        cmd
    } else {
        Command::new("npm")
    };
    cmd.args(args)
        .current_dir(ui_dir)
        .status()
        .with_context(|| format!("npm {} in {}", args.join(" "), ui_dir.display()))
}

fn stage_installer_ui_for_installer(ui_dir: &Path, installer_dir: &Path) -> Result<()> {
    let ui_exe = ui_dir
        .join("src-tauri")
        .join("target")
        .join("release")
        .join("webview-installer-rust.exe");
    if !ui_exe.exists() {
        bail!("installer ui exe not found at {}", ui_exe.display());
    }
    let embedded_dir = installer_dir.join("embedded");
    fs::create_dir_all(&embedded_dir).context("create embedded dir")?;
    let dest = embedded_dir.join("installer-ui.exe");
    fs::copy(&ui_exe, &dest).with_context(|| {
        format!(
            "copy {} -> {}",
            ui_exe.display(),
            dest.display()
        )
    })?;
    Ok(())
}

fn stage_updater_ui_for_updater(ui_dir: &Path, updater_dir: &Path) -> Result<()> {
    let ui_exe = ui_dir.join("target").join("release").join("webview-updater-rust.exe");
    if !ui_exe.exists() {
        bail!("updater ui exe not found at {}", ui_exe.display());
    }
    let embedded_dir = updater_dir.join("embedded");
    fs::create_dir_all(&embedded_dir).context("create embedded dir")?;
    let dest = embedded_dir.join("updater-ui.exe");
    fs::copy(&ui_exe, &dest).with_context(|| {
        format!(
            "copy {} -> {}",
            ui_exe.display(),
            dest.display()
        )
    })?;
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

fn absolutize_path(repo_root: &Path, path: PathBuf) -> PathBuf {
    if path.is_absolute() {
        path
    } else {
        repo_root.join(path)
    }
}

fn find_repo_root() -> Result<PathBuf> {
    let mut candidates = Vec::new();
    if let Ok(cwd) = env::current_dir() {
        candidates.push(cwd);
    }
    if let Ok(exe) = env::current_exe() {
        if let Some(dir) = exe.parent() {
            candidates.push(dir.to_path_buf());
        }
    }

    for start in candidates {
        if let Some(root) = find_upwards(&start) {
            return Ok(root);
        }
    }

    bail!("could not locate repo root (config.toml not found)");
}

fn find_upwards(start: &Path) -> Option<PathBuf> {
    let mut cur = Some(start);
    while let Some(dir) = cur {
        if dir.join("config.toml").exists() && dir.join("installer-rust").is_dir() {
            return Some(dir.to_path_buf());
        }
        cur = dir.parent();
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitize_exe_name_replaces_bad_chars() {
        let name = "My:App*Name?.exe";
        let out = sanitize_exe_name(name);
        assert_eq!(out, "My_App_Name_.exe");
    }

    #[test]
    fn sanitize_exe_name_falls_back_on_empty() {
        let out = sanitize_exe_name("   ");
        assert_eq!(out, "UvesselApp");
    }

    #[test]
    fn require_field_rejects_blank() {
        let err = require_field("name", "   ").unwrap_err();
        assert!(err.to_string().contains("config field name is required"));
    }

    #[test]
    fn parse_arg_finds_value() {
        let args = vec![
            "cmd".to_string(),
            "--out-dir".to_string(),
            "dist".to_string(),
        ];
        assert_eq!(parse_arg(&args, "--out-dir"), Some("dist".to_string()));
    }

    #[test]
    fn absolutize_path_keeps_absolute() {
        let repo = Path::new(r"C:\Repo");
        let out = absolutize_path(repo, PathBuf::from(r"C:\Out"));
        assert_eq!(out, PathBuf::from(r"C:\Out"));
    }

    #[test]
    fn absolutize_path_makes_relative() {
        let repo = Path::new(r"C:\Repo");
        let out = absolutize_path(repo, PathBuf::from("dist"));
        assert_eq!(out, PathBuf::from(r"C:\Repo\dist"));
    }
}
