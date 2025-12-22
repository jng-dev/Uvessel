use anyhow::{Context, Result};
use std::{
    fs,
    path::{Path, PathBuf},
};

const LOG_FILE_NAME: &str = "launcher.log";

pub fn logs_dir(root: &Path) -> PathBuf {
    root.join(".runtime").join("logs")
}

pub fn init(root: &Path) -> Result<PathBuf> {
    let dir = logs_dir(root);
    fs::create_dir_all(&dir).with_context(|| format!("create {}", dir.display()))?;
    let log_path = dir.join(LOG_FILE_NAME);
    let _file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
        .with_context(|| format!("open {}", log_path.display()))?;
    Ok(log_path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn init_creates_log_file() {
        let tmp = tempfile::tempdir().unwrap();
        let log_path = init(tmp.path()).unwrap();
        assert!(log_path.exists());
        assert_eq!(log_path, logs_dir(tmp.path()).join(LOG_FILE_NAME));
    }
}
