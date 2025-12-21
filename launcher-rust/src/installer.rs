use anyhow::{bail, Context, Result};
use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, ExitStatus, Stdio},
};

use crate::{fs_ops, payload, shortcuts, state, uv};

pub fn run(src_root: &Path) -> Result<()> {
    let self_exe = crate::paths::self_path()?;
    let app_name = app_name_from_exe(&self_exe)?;
    let install_root = crate::paths::default_install_root(&app_name)?;

    run_with_deps(
        src_root,
        &install_root,
        &app_name,
        &self_exe,
        uv::ensure_uv,
        |cmd| cmd.status().context("spawn command"),
        |start_menu, name, target, icon| {
            shortcuts::create_start_menu_shortcut(start_menu, name, target, icon)
        },
        |exe| {
            Command::new(exe)
                .spawn()
                .context("launch installed exe")?;
            Ok(())
        },
    )
}

pub fn run_with_deps(
    src_root: &Path,
    install_root: &Path,
    app_name: &str,
    self_exe: &Path,
    ensure_uv_fn: impl Fn(&Path) -> Result<()>,
    mut exec: impl FnMut(&mut Command) -> Result<ExitStatus>,
    create_shortcut_fn: impl Fn(&Path, &str, &Path, Option<&Path>) -> Result<PathBuf>,
    launch_fn: impl Fn(&Path) -> Result<()>,
) -> Result<()> {
    fs::create_dir_all(install_root)
        .with_context(|| format!("create {}", install_root.display()))?;

    let state_path = state::state_path(install_root);
    let existing_state = if state_path.exists() {
        Some(state::read_state(&state_path)?)
    } else {
        None
    };
    let same_version = existing_state
        .as_ref()
        .map(|st| st.launcher_version == crate::config::VERSION)
        .unwrap_or(false);

    let dest_exe = install_root.join(format!("{app_name}.exe"));
    fs_ops::copy_file_with_retry(self_exe, &dest_exe, 5)?;

    if same_version {
        launch_fn(&dest_exe)?;
        return Ok(());
    }

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

    let icon = copy_icon_if_present(src_root, install_root)?;

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

    ensure_console_visible();

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

fn app_name_from_exe(exe: &Path) -> Result<String> {
    let stem = exe.file_stem().and_then(|s| s.to_str()).unwrap_or("");
    if stem.is_empty() {
        bail!("executable name is empty");
    }
    Ok(stem.to_string())
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

fn copy_icon_if_present(src_root: &Path, install_root: &Path) -> Result<Option<PathBuf>> {
    let assets_dir = src_root.join("assets");
    if !assets_dir.exists() {
        return Ok(None);
    }
    let mut ico_paths: Vec<PathBuf> = fs::read_dir(&assets_dir)
        .with_context(|| format!("read_dir {}", assets_dir.display()))?
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| path.extension().map(|e| e.eq_ignore_ascii_case("ico")).unwrap_or(false))
        .collect();
    ico_paths.sort();
    let Some(src_ico) = ico_paths.first() else {
        return Ok(None);
    };
    let file_name = src_ico
        .file_name()
        .context("ico file has no name")?;
    let dest_ico = install_root.join(file_name);
    fs_ops::copy_file_with_retry(src_ico, &dest_ico, 5)?;
    Ok(Some(dest_ico))
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

fn ensure_console_visible() {
    #[cfg(windows)]
    unsafe {
        let _ = windows_sys::Win32::System::Console::AllocConsole();
    }
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
