use anyhow::{bail, Context, Result};
use semver::Version;
use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, ExitStatus, Stdio},
};

use crate::{fs_ops, payload, shortcuts, shim_payload, state, ui_payload, uv};

pub fn run(root: &Path) -> Result<()> {
    let app_name = app_name_from_config();
    let install_root = crate::paths::default_install_root(&app_name)?;

    let done_marker = create_done_marker_path();
    let mut ui_child = match launch_installer_ui(&app_name, Some(&done_marker)) {
        Ok(child) => child,
        Err(err) => {
            eprintln!("warning: failed to launch installer ui: {err}");
            None
        }
    };

    let mut pending_launch: Option<PathBuf> = None;
    let result = run_with_deps(
        root,
        &install_root,
        &app_name,
        uv::ensure_uv,
        |cmd| cmd.status().context("spawn command"),
        |start_menu, name, target, icon| {
            shortcuts::create_start_menu_shortcut(start_menu, name, target, icon)
        },
        |exe| {
            pending_launch = Some(exe.to_path_buf());
            Ok(())
        },
    );

    if result.is_ok() {
        let _ = std::fs::write(&done_marker, "done");
    }

    if let Some(child) = ui_child.as_mut() {
        let _ = child.wait();
    }

    if result.is_ok() {
        if let Some(exe) = pending_launch {
            Command::new(&exe)
                .spawn()
                .with_context(|| format!("launch installed exe {}", exe.display()))?;
        }
    }

    result
}

pub fn run_with_deps(
    _root: &Path,
    install_root: &Path,
    app_name: &str,
    ensure_uv_fn: impl Fn(&Path) -> Result<()>,
    mut exec: impl FnMut(&mut Command) -> Result<ExitStatus>,
    create_shortcut_fn: impl Fn(&Path, &str, &Path, Option<&Path>) -> Result<PathBuf>,
    mut launch_fn: impl FnMut(&Path) -> Result<()>,
) -> Result<()> {
    fs::create_dir_all(install_root)
        .with_context(|| format!("create {}", install_root.display()))?;

    let state_path = state::state_path(install_root);
    let existing_state = if state_path.exists() {
        Some(state::read_state(&state_path)?)
    } else {
        None
    };
    let version_relation = existing_state
        .as_ref()
        .map(|st| compare_versions(&st.launcher_version, crate::config::VERSION))
        .unwrap_or(VersionRelation::Unknown);

    let dest_exe = install_root.join(format!("{app_name}.exe"));
    match version_relation {
        VersionRelation::Same => {
            launch_fn(&dest_exe)?;
            return Ok(());
        }
        VersionRelation::Older => {
            bail!(
                "installed version {} is newer than {}",
                existing_state
                    .as_ref()
                    .map(|st| st.launcher_version.as_str())
                    .unwrap_or("unknown"),
                crate::config::VERSION
            );
        }
        VersionRelation::Newer | VersionRelation::Unknown => {}
    }

    write_shim_exe(&dest_exe)?;

    if existing_state.is_some() {
        remove_app_dir(install_root)?;
        remove_venv_dir(install_root)?;
    }

    payload::install_payload_with_options(
        install_root,
        payload::PayloadOptions {
            skip_existing_data: true,
        },
    )?;

    let icon = resolve_icon_path(install_root);

    ensure_uv_fn(install_root)?;

    let runtime = install_root.join(".runtime");
    ensure_runtime_dirs(&runtime)?;

    let proj = find_project(install_root)?;
    let lock_path = proj.join("uv.lock");
    let lock_mtime = if lock_path.exists() {
        state::file_mtime_unix(&lock_path)?
    } else {
        0
    };

    let uv_exe = install_root.join("uv.exe");
    if !uv_exe.exists() {
        bail!("uv.exe not found after install at {}", uv_exe.display());
    }

    run_with_retry(
        || {
            let mut install = build_uv_cmd(&uv_exe, &proj, &runtime);
            install.arg("python").arg("install");
            if let Some(version) = read_python_version(&proj)? {
                install.arg(version);
            }
            exec(&mut install)
        },
        5,
        "uv python install",
    )?;

    run_with_retry(
        || {
            let mut sync = build_uv_cmd(&uv_exe, &proj, &runtime);
            sync.arg("sync");
            if lock_path.exists() {
                sync.arg("--frozen");
            }
            exec(&mut sync)
        },
        5,
        "uv sync",
    )?;

    cleanup_uv_cache(&runtime)?;

    let start_menu = shortcuts::default_start_menu_dir()?;
    create_shortcut_fn(&start_menu, app_name, &dest_exe, icon.as_deref())?;

    let mut st = state::default_state_for_project(install_root, &proj)?;
    st.lock_mtime_unix = lock_mtime;
    state::write_state(&state::state_path(install_root), &st)?;

    launch_fn(&dest_exe)?;
    Ok(())
}

fn app_name_from_config() -> String {
    let product = crate::config::PRODUCT_NAME.trim();
    if !product.is_empty() {
        return product.to_string();
    }
    let name = crate::config::NAME.trim();
    if !name.is_empty() {
        return name.to_string();
    }
    "UvesselApp".to_string()
}

fn launch_installer_ui(
    app_name: &str,
    done_marker: Option<&Path>,
) -> Result<Option<std::process::Child>> {
    if ui_payload::EMBEDDED_INSTALLER_UI.is_empty() {
        return Ok(None);
    }

    let ui_path = write_installer_ui_exe()?;
    let icon_path = resolve_ui_icon_path()?;

    let mut cmd = Command::new(&ui_path);
    cmd.arg("--name").arg(app_name);
    if let Some(icon_path) = icon_path {
        cmd.arg("--icon").arg(icon_path);
    }
    if let Some(marker) = done_marker {
        cmd.arg("--done-file").arg(marker);
    }
    cmd.stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }

    let child = cmd.spawn().context("spawn installer ui")?;
    Ok(Some(child))
}

fn write_installer_ui_exe() -> Result<PathBuf> {
    let file = tempfile::Builder::new()
        .prefix("uvessel-installer-ui-")
        .suffix(".exe")
        .tempfile()
        .context("create temp installer ui")?;
    let (_, path) = file.keep().context("persist temp installer ui")?;
    fs_ops::write_bytes_with_retry(&path, ui_payload::EMBEDDED_INSTALLER_UI, 3)?;
    Ok(path)
}

fn resolve_ui_icon_path() -> Result<Option<PathBuf>> {
    let icon = crate::config::ICON.trim();
    if icon.is_empty() {
        return Ok(None);
    }
    let icon_path = Path::new(icon);
    if icon_path.is_absolute() {
        return Ok(icon_path.exists().then(|| icon_path.to_path_buf()));
    }
    payload::extract_embedded_file(icon_path)
}

fn create_done_marker_path() -> PathBuf {
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let mut path = std::env::temp_dir();
    path.push(format!("uvessel-install-done-{nonce}.flag"));
    path
}

#[derive(Debug, Clone, Copy)]
enum VersionRelation {
    Same,
    Older,
    Newer,
    Unknown,
}

fn compare_versions(installed: &str, incoming: &str) -> VersionRelation {
    if installed.trim() == incoming.trim() {
        return VersionRelation::Same;
    }
    let installed = Version::parse(installed.trim());
    let incoming = Version::parse(incoming.trim());
    match (installed, incoming) {
        (Ok(installed), Ok(incoming)) => match incoming.cmp(&installed) {
            std::cmp::Ordering::Greater => VersionRelation::Newer,
            std::cmp::Ordering::Less => VersionRelation::Older,
            std::cmp::Ordering::Equal => VersionRelation::Same,
        },
        _ => VersionRelation::Unknown,
    }
}


fn ensure_runtime_dirs(runtime: &Path) -> Result<()> {
    for d in ["cache", "python", "python-bin", "tools", "tool-bin", "venv", "logs"] {
        fs::create_dir_all(runtime.join(d))?;
    }
    Ok(())
}

fn find_project(root: &Path) -> Result<PathBuf> {
    let app_dir = root.join("app");
    let entries = fs::read_dir(&app_dir)
        .with_context(|| format!("read_dir {}", app_dir.display()))?;

    for ent in entries {
        let ent = ent?;
        if !ent.file_type()?.is_dir() {
            continue;
        }
        let candidate = ent.path();
        if candidate.join("pyproject.toml").exists() {
            return Ok(candidate);
        }
    }

    bail!(
        "No project found: expected app/<project>/pyproject.toml under {}",
        app_dir.display()
    )
}

fn build_uv_cmd(uv: &Path, proj: &Path, runtime: &Path) -> Command {
    let mut c = Command::new(uv);
    c.current_dir(proj)
        .envs(uv_env_pairs(runtime))
        .stdin(Stdio::null());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        c.creation_flags(CREATE_NO_WINDOW);
    }
    c
}

fn uv_env_pairs(runtime: &Path) -> Vec<(String, String)> {
    vec![
        ("UV_CACHE_DIR".to_string(), runtime.join("cache").to_string_lossy().to_string()),
        ("UV_PYTHON_INSTALL_DIR".to_string(), runtime.join("python").to_string_lossy().to_string()),
        ("UV_PYTHON_BIN_DIR".to_string(), runtime.join("python-bin").to_string_lossy().to_string()),
        ("UV_PROJECT_ENVIRONMENT".to_string(), runtime.join("venv").to_string_lossy().to_string()),
        ("UV_TOOL_DIR".to_string(), runtime.join("tools").to_string_lossy().to_string()),
        ("UV_TOOL_BIN_DIR".to_string(), runtime.join("tool-bin").to_string_lossy().to_string()),
        ("UV_NO_CONFIG".to_string(), "1".to_string()),
    ]
}

fn read_python_version(proj: &Path) -> Result<Option<String>> {
    let version_path = proj.join(".python-version");
    if !version_path.exists() {
        return Ok(None);
    }
    let contents = fs::read_to_string(&version_path)
        .with_context(|| format!("read {}", version_path.display()))?;
    let version = contents.lines().next().unwrap_or("").trim();
    if version.is_empty() {
        return Ok(None);
    }
    Ok(Some(version.to_string()))
}

fn resolve_icon_path(install_root: &Path) -> Option<PathBuf> {
    let icon = crate::config::ICON.trim();
    if icon.is_empty() {
        return None;
    }
    let icon_path = Path::new(icon);
    if icon_path.is_absolute() {
        return icon_path.exists().then(|| icon_path.to_path_buf());
    }
    let candidate = install_root.join(icon_path);
    candidate.exists().then_some(candidate)
}

fn write_shim_exe(dest_exe: &Path) -> Result<()> {
    if shim_payload::EMBEDDED_SHIM.is_empty() {
        bail!("embedded shim is empty");
    }
    fs_ops::write_bytes_with_retry(dest_exe, shim_payload::EMBEDDED_SHIM, 5)
}

fn remove_app_dir(install_root: &Path) -> Result<()> {
    let app_dir = install_root.join("app");
    if app_dir.exists() {
        fs::remove_dir_all(&app_dir)
            .with_context(|| format!("remove {}", app_dir.display()))?;
    }
    Ok(())
}

fn remove_venv_dir(install_root: &Path) -> Result<()> {
    let venv_dir = install_root.join(".runtime").join("venv");
    if venv_dir.exists() {
        fs::remove_dir_all(&venv_dir)
            .with_context(|| format!("remove {}", venv_dir.display()))?;
    }
    Ok(())
}

fn cleanup_uv_cache(runtime: &Path) -> Result<()> {
    let cache = runtime.join("cache");
    if cache.exists() {
        fs::remove_dir_all(&cache)
            .with_context(|| format!("remove {}", cache.display()))?;
    }
    Ok(())
}

fn run_with_retry(
    mut make_status: impl FnMut() -> Result<ExitStatus>,
    attempts: usize,
    label: &str,
) -> Result<()> {
    let mut delay = std::time::Duration::from_millis(250);
    for i in 0..attempts {
        let status = make_status()?;
        if status.success() {
            return Ok(());
        }
        if i + 1 == attempts {
            eprintln!("warning: {label} failed after {attempts} attempts");
            bail!("{label} failed (exit {:?})", status.code());
        }
        eprintln!(
            "warning: {label} failed (exit {:?}), retrying...",
            status.code()
        );
        std::thread::sleep(delay);
        delay = std::cmp::min(delay * 2, std::time::Duration::from_secs(5));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn app_name_from_exe_rejects_empty() {
        let err = app_name_from_exe(Path::new("")).unwrap_err();
        assert!(err.to_string().contains("executable name is empty"));
    }

    #[test]
    fn uv_env_pairs_include_no_config() {
        let tmp = tempfile::tempdir().unwrap();
        let envs = uv_env_pairs(&tmp.path().join(".runtime"));
        assert!(envs.iter().any(|(k, v)| k == "UV_NO_CONFIG" && v == "1"));
    }

    #[test]
    fn read_python_version_from_file() {
        let tmp = tempfile::tempdir().unwrap();
        let proj = tmp.path();
        fs::write(proj.join(".python-version"), "3.12\n").unwrap();
        let v = read_python_version(proj).unwrap();
        assert_eq!(v.as_deref(), Some("3.12"));
    }
}
