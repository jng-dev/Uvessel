use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::{Duration, UNIX_EPOCH},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value")]
enum EntryPoint {
    #[serde(rename = "python_file")]
    PythonFile(String), // e.g. "main.py"
    #[serde(rename = "module")]
    Module(String),     // e.g. "pkg.__main__"
    #[serde(rename = "command")]
    Command(String),    // e.g. "mycli"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct State {
    project_rel: String, // e.g. "app\\test-project"
    entry: EntryPoint,
    lock_mtime_unix: u64,
}

fn root_dir() -> Result<PathBuf> {
    let exe = std::env::current_exe().context("current_exe")?;
    Ok(exe.parent().context("exe has no parent")?.to_path_buf())
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

    bail!("No project found: expected app/<project>/pyproject.toml under {}", app_dir.display())
}

fn file_mtime_unix(path: &Path) -> Result<u64> {
    let meta = fs::metadata(path).with_context(|| format!("metadata {}", path.display()))?;
    let mtime = meta.modified().context("modified time")?;
    Ok(mtime
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::from_secs(0))
        .as_secs())
}

fn uv_cmd(uv: &Path, proj: &Path, runtime: &Path) -> Command {
    let mut c = Command::new(uv);
    c.current_dir(proj)
        .env("UV_CACHE_DIR", runtime.join("cache"))
        .env("UV_PYTHON_INSTALL_DIR", runtime.join("python"))
        .env("UV_PYTHON_BIN_DIR", runtime.join("python-bin"))
        .env("UV_PROJECT_ENVIRONMENT", runtime.join("venv"))
        .env("UV_TOOL_DIR", runtime.join("tools"))
        .env("UV_TOOL_BIN_DIR", runtime.join("tool-bin"))
        .env("UV_NO_CONFIG", "1")
        .stdin(Stdio::null());
    c
}

fn run_with_retry(mut make_cmd: impl FnMut() -> Command, attempts: usize) -> Result<()> {
    let mut delay = Duration::from_millis(250);
    for i in 0..attempts {
        let status = make_cmd().status().context("spawn command")?;
        if status.success() {
            return Ok(());
        }
        if i + 1 == attempts {
            bail!("command failed after {attempts} attempts (last exit {:?})", status.code());
        }
        std::thread::sleep(delay);
        delay = std::cmp::min(delay * 2, Duration::from_secs(5));
    }
    Ok(())
}

fn default_state_for_project(root: &Path, proj: &Path) -> Result<State> {
    // store relative path like "app\\test-project"
    let rel = proj.strip_prefix(root).unwrap_or(proj);
    let rel_str = rel.to_string_lossy().to_string();

    // default entry: main.py if present
    let main_py = proj.join("main.py");
    if !main_py.exists() {
        bail!("Default entrypoint main.py not found at {}", main_py.display());
    }

    Ok(State {
        project_rel: rel_str,
        entry: EntryPoint::PythonFile("main.py".to_string()),
        lock_mtime_unix: 0,
    })
}

fn read_or_init_state(state_path: &Path, root: &Path, proj: &Path) -> Result<State> {
    if state_path.exists() {
        let s = fs::read_to_string(state_path).context("read state.json")?;
        return Ok(serde_json::from_str(&s).context("parse state.json")?);
    }
    let st = default_state_for_project(root, proj)?;
    fs::write(state_path, serde_json::to_string_pretty(&st).unwrap()).context("write state.json")?;
    Ok(st)
}

fn main() -> Result<()> {
    let root = root_dir()?;
    let uv = root.join("uv.exe");
    if !uv.exists() {
        bail!("uv.exe not found next to launcher at {}", uv.display());
    }

    let runtime = root.join(".runtime");
    ensure_runtime_dirs(&runtime)?;

    let proj = find_project(&root)?;
    let lock_path = proj.join("uv.lock");

    let state_path = runtime.join("state.json");
    let mut state = read_or_init_state(&state_path, &root, &proj)?;

    // Re-resolve project from state (so config persists)
    let proj_from_state = root.join(&state.project_rel);
    if !proj_from_state.join("pyproject.toml").exists() {
        bail!(
            "Configured project is invalid: {} (missing pyproject.toml)",
            proj_from_state.display()
        );
    }

    let lock_mtime = if lock_path.exists() { file_mtime_unix(&lock_path)? } else { 0 };

    let venv_cfg = runtime.join("venv").join("pyvenv.cfg");
    let need_bootstrap = !venv_cfg.exists() || (lock_mtime != 0 && lock_mtime != state.lock_mtime_unix);

    if need_bootstrap {
        run_with_retry(
            || {
                let mut c = uv_cmd(&uv, &proj_from_state, &runtime);
                c.arg("python").arg("install");
                c
            },
            5,
        )?;

        run_with_retry(
            || {
                let mut c = uv_cmd(&uv, &proj_from_state, &runtime);
                c.arg("sync");
                if lock_path.exists() {
                    c.arg("--frozen");
                }
                c
            },
            5,
        )?;

        state.lock_mtime_unix = lock_mtime;
        fs::write(&state_path, serde_json::to_string_pretty(&state).unwrap())
            .context("write updated state.json")?;
    }

    // Run entrypoint
    let mut run = uv_cmd(&uv, &proj_from_state, &runtime);
    run.arg("run");
    match &state.entry {
        EntryPoint::PythonFile(f) => run.arg("python").arg(f),
        EntryPoint::Module(m) => run.arg("python").arg("-m").arg(m),
        EntryPoint::Command(cmd) => run.arg(cmd),
    };

    let status = run.status().context("run entrypoint")?;
    std::process::exit(status.code().unwrap_or(1));
}