use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
    time::{Duration, UNIX_EPOCH},
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", content = "value")]
pub enum EntryPoint {
    #[serde(rename = "python_file")]
    PythonFile(String),
    #[serde(rename = "module")]
    Module(String),
    #[serde(rename = "command")]
    Command(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct State {
    pub project_rel: String,
    pub entry: EntryPoint,
    pub lock_mtime_unix: u64,
    pub installed: bool,
}

pub fn state_path(root: &Path) -> PathBuf {
    root.join(".runtime").join("state.json")
}

pub fn state_exists(root: &Path) -> bool {
    state_path(root).exists()
}

pub fn file_mtime_unix(path: &Path) -> Result<u64> {
    let meta = fs::metadata(path).with_context(|| format!("metadata {}", path.display()))?;
    let mtime = meta.modified().context("modified time")?;
    Ok(mtime
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::from_secs(0))
        .as_secs())
}

pub fn default_state_for_project(root: &Path, proj: &Path) -> Result<State> {
    let rel = proj.strip_prefix(root).unwrap_or(proj);
    let rel_str = rel.to_string_lossy().to_string();

    let main_py = proj.join("main.py");
    if !main_py.exists() {
        bail!("Default entrypoint main.py not found at {}", main_py.display());
    }

    Ok(State {
        project_rel: rel_str,
        entry: EntryPoint::PythonFile("main.py".to_string()),
        lock_mtime_unix: 0,
        installed: true,
    })
}

pub fn read_state(state_path: &Path) -> Result<State> {
    let s = fs::read_to_string(state_path).context("read state.json")?;
    Ok(serde_json::from_str(&s).context("parse state.json")?)
}

pub fn write_state(state_path: &Path, state: &State) -> Result<()> {
    if let Some(parent) = state_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    let contents = serde_json::to_string_pretty(state).context("serialize state.json")?;
    fs::write(state_path, contents).context("write state.json")?;
    Ok(())
}

pub fn read_or_init_state(state_path: &Path, root: &Path, proj: &Path) -> Result<State> {
    if state_path.exists() {
        return read_state(state_path);
    }
    let st = default_state_for_project(root, proj)?;
    write_state(state_path, &st)?;
    Ok(st)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serde_roundtrip_state() {
        let state = State {
            project_rel: "app\\proj".to_string(),
            entry: EntryPoint::PythonFile("main.py".to_string()),
            lock_mtime_unix: 123,
            installed: true,
        };
        let s = serde_json::to_string(&state).unwrap();
        let out: State = serde_json::from_str(&s).unwrap();
        assert_eq!(state, out);
    }

    #[test]
    fn read_or_init_creates_state() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let proj = root.join("app").join("proj");
        fs::create_dir_all(&proj).unwrap();
        fs::write(proj.join("main.py"), "print('hi')").unwrap();

        let st_path = state_path(root);
        let state = read_or_init_state(&st_path, root, &proj).unwrap();

        assert!(st_path.exists());
        assert_eq!(state.project_rel, PathBuf::from("app").join("proj").to_string_lossy());
        assert_eq!(
            state,
            State {
                project_rel: PathBuf::from("app").join("proj").to_string_lossy().to_string(),
                entry: EntryPoint::PythonFile("main.py".to_string()),
                lock_mtime_unix: 0,
                installed: true,
            }
        );
    }
}
