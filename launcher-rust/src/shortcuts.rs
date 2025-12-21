use anyhow::{bail, Context, Result};
use std::{
    path::{Path, PathBuf},
    process::Command,
};

pub fn default_start_menu_dir() -> Result<PathBuf> {
    let appdata = std::env::var("APPDATA").context("APPDATA not set")?;
    Ok(PathBuf::from(appdata).join("Microsoft").join("Windows").join("Start Menu").join("Programs"))
}

pub fn shortcut_path(start_menu_dir: &Path, name: &str) -> Result<PathBuf> {
    if name.is_empty() {
        bail!("shortcut name is empty");
    }
    Ok(start_menu_dir.join(format!("{name}.lnk")))
}

pub fn create_start_menu_shortcut(
    start_menu_dir: &Path,
    name: &str,
    target: &Path,
    icon: Option<&Path>,
) -> Result<PathBuf> {
    let lnk_path = shortcut_path(start_menu_dir, name)?;
    std::fs::create_dir_all(start_menu_dir)
        .with_context(|| format!("create {}", start_menu_dir.display()))?;

    let lnk = ps_quote(&lnk_path.display().to_string());
    let tgt = ps_quote(&target.display().to_string());
    let icon = icon.map(|p| ps_quote(&p.display().to_string()));

    let mut script = format!(
        "$WshShell = New-Object -ComObject WScript.Shell; \
         $Shortcut = $WshShell.CreateShortcut({lnk}); \
         $Shortcut.TargetPath = {tgt}; "
    );
    if let Some(icon_path) = icon {
        script.push_str(&format!("$Shortcut.IconLocation = {icon_path}; "));
    }
    script.push_str("$Shortcut.Save();");

    let status = Command::new("powershell")
        .arg("-NoProfile")
        .arg("-Command")
        .arg(script)
        .status()
        .context("run powershell")?;

    if !status.success() {
        bail!("failed to create shortcut (exit {:?})", status.code());
    }

    Ok(lnk_path)
}

fn ps_quote(value: &str) -> String {
    let escaped = value.replace('\'', "''");
    format!("'{}'", escaped)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shortcut_path_adds_lnk() {
        let base = PathBuf::from(r"C:\StartMenu");
        let out = shortcut_path(&base, "MyApp").unwrap();
        assert_eq!(out, base.join("MyApp.lnk"));
    }

    #[test]
    fn shortcut_path_rejects_empty_name() {
        let base = PathBuf::from(r"C:\StartMenu");
        let err = shortcut_path(&base, "").unwrap_err();
        assert!(err.to_string().contains("shortcut name is empty"));
    }
}
