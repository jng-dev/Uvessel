use anyhow::{bail, Context, Result};
use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, ExitStatus, Stdio},
};

use crate::state::{self, EntryPoint, State};

pub fn run(root: &Path) -> Result<()> {
    run_with_executor(root, |cmd| cmd.status().context("spawn command"))
}

pub fn run_with_executor(
    root: &Path,
    mut exec: impl FnMut(&mut Command) -> Result<ExitStatus>,
) -> Result<()> {
    if let UpdateDecision::ExitForUpdate = maybe_run_updater(root)? {
        return Ok(());
    }

    let uv = root.join("uv.exe");
    if !uv.exists() {
        bail!("uv.exe not found next to launcher at {}", uv.display());
    }

    let runtime = root.join(".runtime");
    ensure_runtime_dirs(&runtime)?;

    let state_path = state::state_path(root);
    let mut st = state::read_state(&state_path)?;
    let proj = resolve_project(root, &st)?;

    let lock_path = proj.join("uv.lock");
    let lock_mtime = if lock_path.exists() {
        state::file_mtime_unix(&lock_path)?
    } else {
        0
    };

    if needs_bootstrap(&runtime, lock_mtime, st.lock_mtime_unix) {
        run_with_retry(
            || {
                let mut install = build_uv_cmd(&uv, &proj, &runtime);
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
                let mut sync = build_uv_cmd(&uv, &proj, &runtime);
                sync.arg("sync");
                if lock_path.exists() {
                    sync.arg("--frozen");
                }
                exec(&mut sync)
            },
            5,
            "uv sync",
        )?;

        st.lock_mtime_unix = lock_mtime;
        state::write_state(&state_path, &st)?;

        cleanup_uv_cache(&runtime)?;
    }

    let mut run_cmd = build_uv_cmd(&uv, &proj, &runtime);
    run_cmd.arg("run");
    match &st.entry {
        EntryPoint::PythonFile(f) => run_cmd.arg("python").arg(f),
        EntryPoint::Module(m) => run_cmd.arg("python").arg("-m").arg(m),
        EntryPoint::Command(cmd) => run_cmd.arg(cmd),
    };

    let status = exec(&mut run_cmd)?;
    if !status.success() {
        bail!("entrypoint failed (exit {:?})", status.code());
    }

    Ok(())
}

fn ensure_runtime_dirs(runtime: &Path) -> Result<()> {
    for d in ["cache", "python", "python-bin", "tools", "tool-bin", "venv", "logs"] {
        fs::create_dir_all(runtime.join(d))?;
    }
    Ok(())
}

fn resolve_project(root: &Path, st: &State) -> Result<PathBuf> {
    let proj = root.join(&st.project_rel);
    let pyproject = proj.join("pyproject.toml");
    if !pyproject.exists() {
        bail!(
            "Configured project is invalid: {} (missing pyproject.toml)",
            proj.display()
        );
    }
    Ok(proj)
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

enum UpdateDecision {
    Continue,
    ExitForUpdate,
}

fn maybe_run_updater(root: &Path) -> Result<UpdateDecision> {
    let updater = root.join("updater.exe");
    if !updater.exists() {
        return Ok(UpdateDecision::Continue);
    }

    let mut cmd = Command::new(&updater);
    cmd.stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }

    let status = cmd.status().context("run updater")?;
    if let Some(code) = status.code() {
        if code == 10 {
            return Ok(UpdateDecision::ExitForUpdate);
        }
        if code != 0 {
            eprintln!("warning: updater exited with {code}");
        }
    } else if !status.success() {
        eprintln!("warning: updater exit status unknown");
    }

    Ok(UpdateDecision::Continue)
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

fn needs_bootstrap(runtime: &Path, lock_mtime: u64, state_lock_mtime: u64) -> bool {
    let venv_cfg = runtime.join("venv").join("pyvenv.cfg");
    if !venv_cfg.exists() {
        return true;
    }
    lock_mtime != 0 && lock_mtime != state_lock_mtime
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
    fn needs_bootstrap_when_venv_missing() {
        let tmp = tempfile::tempdir().unwrap();
        let runtime = tmp.path().join(".runtime");
        fs::create_dir_all(&runtime).unwrap();
        assert!(needs_bootstrap(&runtime, 0, 0));
    }

    #[test]
    fn needs_bootstrap_when_lock_changed() {
        let tmp = tempfile::tempdir().unwrap();
        let runtime = tmp.path().join(".runtime");
        fs::create_dir_all(runtime.join("venv")).unwrap();
        fs::write(runtime.join("venv").join("pyvenv.cfg"), "cfg").unwrap();
        assert!(needs_bootstrap(&runtime, 10, 5));
        assert!(!needs_bootstrap(&runtime, 10, 10));
    }

    #[test]
    fn uv_env_pairs_include_no_config() {
        let tmp = tempfile::tempdir().unwrap();
        let runtime = tmp.path().join(".runtime");
        let envs = uv_env_pairs(&runtime);
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
