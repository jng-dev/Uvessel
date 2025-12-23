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
    let log_path = create_log_path();
    let _ = init_log_file(&log_path);
    let launch_marker = create_launch_marker_path();
    let update_mode = detect_update_mode(&install_root)?;
    let mut ui_child = match launch_installer_ui(
        &app_name,
        Some(&done_marker),
        Some(&log_path),
        Some(&launch_marker),
        update_mode,
    ) {
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
        |cmd| exec_with_log(cmd, Some(&log_path)),
        |start_menu, name, target, icon| {
            shortcuts::create_start_menu_shortcut(start_menu, name, target, icon)
        },
        |exe| {
            pending_launch = Some(exe.to_path_buf());
            Ok(())
        },
        Some(&log_path),
    );

    if result.is_ok() {
        let _ = std::fs::write(&done_marker, "ok");
    } else {
        let _ = std::fs::write(&done_marker, "fail");
    }

    if let Some(child) = ui_child.as_mut() {
        let _ = child.wait();
    }

    let should_launch = std::fs::metadata(&launch_marker).is_ok();
    let _ = std::fs::remove_file(&log_path);
    let _ = std::fs::remove_file(&launch_marker);

    if result.is_ok() && should_launch {
        if let Some(exe) = pending_launch {
            Command::new(&exe)
                .spawn()
                .with_context(|| format!("launch installed exe {}", exe.display()))?;
        }
    }

    result
}

pub fn run_from_args(root: &Path) -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if let Some(target) = arg_value(&args, "--finalize-uninstall") {
        let app_name = arg_value(&args, "--app-name");
        return finalize_uninstall(Path::new(&target), app_name.as_deref());
    }

    if args.iter().any(|arg| arg == "--uninstall") || exe_name_is_uninstaller()? {
        return run_uninstall(root);
    }

    run(root)
}

pub fn run_with_deps(
    _root: &Path,
    install_root: &Path,
    app_name: &str,
    ensure_uv_fn: impl Fn(&Path) -> Result<()>,
    mut exec: impl FnMut(&mut Command) -> Result<ExitStatus>,
    create_shortcut_fn: impl Fn(&Path, &str, &Path, Option<&Path>) -> Result<PathBuf>,
    mut launch_fn: impl FnMut(&Path) -> Result<()>,
    log_path: Option<&Path>,
) -> Result<()> {
    let install_root_existed = install_root.exists();
    log_line(log_path, &format!("Starting install for {app_name}"))?;
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
            log_line(log_path, "Installed version matches, launching app")?;
            let icon = resolve_icon_path(install_root);
            if let Ok(start_menu) = shortcuts::default_start_menu_dir() {
                let uninstall_name = format!("Uninstall {app_name}");
                if let Ok(uninstall_exe) = ensure_uninstaller(install_root) {
                    let _ = shortcuts::create_start_menu_shortcut(
                        &start_menu,
                        &uninstall_name,
                        &uninstall_exe,
                        icon.as_deref(),
                    );
                }
            }
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

    let mut backup = if existing_state.is_some() {
        log_line(log_path, "Existing install detected, preparing backup")?;
        Some(UpgradeBackup::create(install_root)?)
    } else {
        None
    };

    let install_result = (|| -> Result<()> {
        log_line(log_path, "Writing launcher shim")?;
        write_shim_exe(&dest_exe)?;

        log_line(log_path, "Extracting payload")?;
        payload::install_payload_with_options(
            install_root,
            payload::PayloadOptions {
                skip_existing_data: true,
            },
        )?;

        let icon = resolve_icon_path(install_root);

        log_line(log_path, "Ensuring uv")?;
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

        log_line(log_path, "Cleaning runtime cache")?;
        cleanup_uv_cache(&runtime)?;

        let start_menu = shortcuts::default_start_menu_dir()?;
        create_shortcut_fn(&start_menu, app_name, &dest_exe, icon.as_deref())?;
        let uninstall_exe = ensure_uninstaller(install_root)?;
        let uninstall_name = format!("Uninstall {app_name}");
        shortcuts::create_start_menu_shortcut(
            &start_menu,
            &uninstall_name,
            &uninstall_exe,
            icon.as_deref(),
        )?;

        let mut st = state::default_state_for_project(install_root, &proj)?;
        st.lock_mtime_unix = lock_mtime;
        state::write_state(&state::state_path(install_root), &st)?;

        log_line(log_path, "Launching application")?;
        launch_fn(&dest_exe)?;
        Ok(())
    })();

    match install_result {
        Ok(()) => {
            log_line(log_path, "Install completed successfully")?;
            if let Some(mut backup) = backup.take() {
                backup.cleanup()?;
            }
            Ok(())
        }
        Err(err) => {
            let _ = log_line(log_path, &format!("Install failed: {err}"));
            if let Some(mut backup) = backup.take() {
                let _ = backup.restore();
            } else if !install_root_existed {
                let _ = fs::remove_dir_all(install_root);
            }
            Err(err)
        }
    }
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

fn detect_update_mode(install_root: &Path) -> Result<bool> {
    let state_path = state::state_path(install_root);
    if !state_path.exists() {
        return Ok(false);
    }
    let existing_state = state::read_state(&state_path)?;
    let relation = compare_versions(&existing_state.launcher_version, crate::config::VERSION);
    Ok(!matches!(relation, VersionRelation::Same))
}

fn launch_installer_ui(
    app_name: &str,
    done_marker: Option<&Path>,
    log_path: Option<&Path>,
    launch_marker: Option<&Path>,
    update_mode: bool,
) -> Result<Option<std::process::Child>> {
    if ui_payload::EMBEDDED_INSTALLER_UI.is_empty() {
        return Ok(None);
    }

    let ui_path = write_installer_ui_exe()?;
    let icon_path = resolve_ui_icon_path()?;

    let mut cmd = Command::new(&ui_path);
    cmd.arg("--name").arg(app_name);
    cmd.arg("--version").arg(crate::config::VERSION);
    if let Some(icon_path) = icon_path {
        cmd.arg("--icon").arg(icon_path);
    }
    if let Some(marker) = done_marker {
        cmd.arg("--done-file").arg(marker);
    }
    if let Some(log_path) = log_path {
        cmd.arg("--log-file").arg(log_path);
    }
    if let Some(launch_marker) = launch_marker {
        cmd.arg("--launch-file").arg(launch_marker);
    }
    if update_mode {
        cmd.arg("--mode").arg("update");
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

fn run_uninstall(_root: &Path) -> Result<()> {
    let app_name = app_name_from_config();
    let install_root = crate::paths::default_install_root(&app_name)?;
    if !install_root.exists() {
        return Ok(());
    }

    let current_exe = std::env::current_exe().context("resolve current exe")?;
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let temp_exe = std::env::temp_dir().join(format!("uvessel-uninstall-{nonce}.exe"));
    fs::copy(&current_exe, &temp_exe)
        .with_context(|| format!("copy {} -> {}", current_exe.display(), temp_exe.display()))?;

    let mut cmd = Command::new(&temp_exe);
    cmd.arg("--finalize-uninstall")
        .arg(&install_root)
        .arg("--app-name")
        .arg(&app_name)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }
    cmd.spawn().context("spawn uninstall finalize")?;
    Ok(())
}

fn finalize_uninstall(install_root: &Path, app_name: Option<&str>) -> Result<()> {
    if let Some(name) = app_name {
        if let Ok(start_menu) = shortcuts::default_start_menu_dir() {
            let _ = shortcuts::remove_start_menu_shortcut(&start_menu, name);
            let uninstall_name = format!("Uninstall {name}");
            let _ = shortcuts::remove_start_menu_shortcut(&start_menu, &uninstall_name);
        }
    }

    if install_root.exists() {
        fs::remove_dir_all(install_root)
            .with_context(|| format!("remove {}", install_root.display()))?;
    }
    Ok(())
}

fn exe_name_is_uninstaller() -> Result<bool> {
    let exe = std::env::current_exe().context("resolve current exe")?;
    let name = exe
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or_default()
        .to_lowercase();
    Ok(name.contains("uninstall"))
}

fn arg_value(args: &[String], key: &str) -> Option<String> {
    args.iter()
        .position(|arg| arg == key)
        .and_then(|idx| args.get(idx + 1))
        .cloned()
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

fn create_log_path() -> PathBuf {
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let mut path = std::env::temp_dir();
    path.push(format!("uvessel-install-log-{nonce}.txt"));
    path
}

fn create_launch_marker_path() -> PathBuf {
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let mut path = std::env::temp_dir();
    path.push(format!("uvessel-install-launch-{nonce}.flag"));
    path
}

fn init_log_file(path: &Path) -> Result<()> {
    fs::write(path, "installer log start\n")
        .with_context(|| format!("write {}", path.display()))?;
    Ok(())
}

fn log_line(path: Option<&Path>, line: &str) -> Result<()> {
    let Some(path) = path else {
        return Ok(());
    };
    use std::io::Write;
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .with_context(|| format!("open {}", path.display()))?;
    writeln!(file, "{line}")?;
    Ok(())
}

fn exec_with_log(cmd: &mut Command, log_path: Option<&Path>) -> Result<ExitStatus> {
    if let Some(log_path) = log_path {
        let line = format!("> {}", format_command(cmd));
        let _ = log_line(Some(log_path), &line);
    }
    let output = cmd.output().context("spawn command")?;
    if let Some(log_path) = log_path {
        if !output.stdout.is_empty() {
            let text = String::from_utf8_lossy(&output.stdout);
            let _ = log_line(Some(log_path), text.trim_end());
        }
        if !output.stderr.is_empty() {
            let text = String::from_utf8_lossy(&output.stderr);
            let _ = log_line(Some(log_path), text.trim_end());
        }
        let _ = log_line(
            Some(log_path),
            &format!(
                "exit status: {}",
                output.status.code().map(|c| c.to_string()).unwrap_or_else(|| "unknown".to_string())
            ),
        );
    }
    Ok(output.status)
}

fn format_command(cmd: &Command) -> String {
    let program = cmd.get_program().to_string_lossy();
    let args = cmd
        .get_args()
        .map(|arg| arg.to_string_lossy())
        .collect::<Vec<_>>()
        .join(" ");
    if args.is_empty() {
        program.to_string()
    } else {
        format!("{program} {args}")
    }
}

fn ensure_uninstaller(install_root: &Path) -> Result<PathBuf> {
    let dest = install_root.join("uninstaller.exe");
    let current_exe = std::env::current_exe().context("resolve current exe")?;
    if dest != current_exe {
        fs::copy(&current_exe, &dest)
            .with_context(|| format!("copy {} -> {}", current_exe.display(), dest.display()))?;
    }
    Ok(dest)
}

struct UpgradeBackup {
    app_backup: Option<PathBuf>,
    venv_backup: Option<PathBuf>,
}

impl UpgradeBackup {
    fn create(install_root: &Path) -> Result<Self> {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let app_dir = install_root.join("app");
        let venv_dir = install_root.join(".runtime").join("venv");
        let app_backup = if app_dir.exists() {
            let backup = install_root.join(format!("app.backup.{nonce}"));
            fs::rename(&app_dir, &backup)
                .with_context(|| format!("rename {} -> {}", app_dir.display(), backup.display()))?;
            Some(backup)
        } else {
            None
        };
        let venv_backup = if venv_dir.exists() {
            let backup = install_root.join(format!("venv.backup.{nonce}"));
            fs::rename(&venv_dir, &backup)
                .with_context(|| format!("rename {} -> {}", venv_dir.display(), backup.display()))?;
            Some(backup)
        } else {
            None
        };
        Ok(Self {
            app_backup,
            venv_backup,
        })
    }

    fn restore(&mut self) -> Result<()> {
        if let Some(backup) = self.app_backup.take() {
            let target = backup.parent().unwrap_or_else(|| Path::new(".")).join("app");
            if !target.exists() {
                fs::rename(&backup, &target).with_context(|| {
                    format!("restore {} -> {}", backup.display(), target.display())
                })?;
            }
        }
        if let Some(backup) = self.venv_backup.take() {
            let target = backup.parent().unwrap_or_else(|| Path::new(".")).join(".runtime").join("venv");
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent)?;
            }
            if !target.exists() {
                fs::rename(&backup, &target).with_context(|| {
                    format!("restore {} -> {}", backup.display(), target.display())
                })?;
            }
        }
        Ok(())
    }

    fn cleanup(&mut self) -> Result<()> {
        if let Some(backup) = self.app_backup.take() {
            if backup.exists() {
                fs::remove_dir_all(&backup)
                    .with_context(|| format!("remove {}", backup.display()))?;
            }
        }
        if let Some(backup) = self.venv_backup.take() {
            if backup.exists() {
                fs::remove_dir_all(&backup)
                    .with_context(|| format!("remove {}", backup.display()))?;
            }
        }
        Ok(())
    }
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
