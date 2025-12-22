use anyhow::{Context, Result};
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
    #[serde(default)]
    pub launcher_version: String,
}

pub fn state_path(root: &Path) -> PathBuf {
    root.join(".runtime").join("state.json")
}

pub fn file_mtime_unix(path: &Path) -> Result<u64> {
    let meta = fs::metadata(path).with_context(|| format!("metadata {}", path.display()))?;
    let mtime = meta.modified().context("modified time")?;
    Ok(mtime
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::from_secs(0))
        .as_secs())
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
            launcher_version: "1.2.3".to_string(),
        };
        let s = serde_json::to_string(&state).unwrap();
        let out: State = serde_json::from_str(&s).unwrap();
        assert_eq!(state, out);
    }
}
