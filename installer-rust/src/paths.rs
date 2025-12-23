use anyhow::{bail, Context, Result};
use std::path::{Path, PathBuf};
use crate::config;

pub fn self_path() -> Result<PathBuf> {
    Ok(std::env::current_exe().context("current_exe")?)
}

pub fn root_dir() -> Result<PathBuf> {
    if let Ok(dev_root) = std::env::var("UVESSEL_ROOT") {
        return Ok(PathBuf::from(dev_root));
    }
    let exe = self_path()?;
    Ok(exe.parent().context("exe has no parent")?.to_path_buf())
}

pub fn default_install_root(app_name: &str) -> Result<PathBuf> {
    if app_name.is_empty() {
        bail!("app_name is empty");
    }
    let custom = config::INSTALL_DIR.trim();
    if !custom.is_empty() {
        let base = PathBuf::from(custom);
        if base.is_absolute() {
            return Ok(base.join(app_name));
        }
        let local = std::env::var("LOCALAPPDATA").context("LOCALAPPDATA not set")?;
        return Ok(PathBuf::from(local).join("Uvessel").join(base).join(app_name));
    }
    let local = std::env::var("LOCALAPPDATA").context("LOCALAPPDATA not set")?;
    Ok(PathBuf::from(local).join("Uvessel").join(app_name))
}

#[cfg(test)]
pub fn runtime_dir(root: &Path) -> PathBuf {
    root.join(".runtime")
}

#[cfg(test)]
pub fn app_dir(root: &Path) -> PathBuf {
    root.join("app")
}

#[cfg(test)]
pub fn uv_paths(root: &Path) -> (PathBuf, PathBuf, PathBuf) {
    (
        root.join("uv.exe"),
        root.join("uvx.exe"),
        root.join("uvw.exe"),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    static ENV_MUTEX: Mutex<()> = Mutex::new(());

    #[test]
    fn root_dir_prefers_env() {
        let _guard = ENV_MUTEX.lock().unwrap();
        let prior = std::env::var("UVESSEL_ROOT").ok();

        std::env::set_var("UVESSEL_ROOT", r"C:\Temp\UvesselRoot");
        let root = root_dir().unwrap();
        assert_eq!(root, PathBuf::from(r"C:\Temp\UvesselRoot"));

        if let Some(v) = prior {
            std::env::set_var("UVESSEL_ROOT", v);
        } else {
            std::env::remove_var("UVESSEL_ROOT");
        }
    }

    #[test]
    fn default_install_root_uses_localappdata() {
        let _guard = ENV_MUTEX.lock().unwrap();
        let prior = std::env::var("LOCALAPPDATA").ok();

        let tmp = tempfile::tempdir().unwrap();
        std::env::set_var("LOCALAPPDATA", tmp.path());

        let root = default_install_root("MyApp").unwrap();
        assert_eq!(
            root,
            tmp.path().join("Uvessel").join("MyApp")
        );

        if let Some(v) = prior {
            std::env::set_var("LOCALAPPDATA", v);
        } else {
            std::env::remove_var("LOCALAPPDATA");
        }
    }

    #[test]
    fn runtime_dir_is_dot_runtime() {
        let root = PathBuf::from(r"C:\Apps\MyApp");
        assert_eq!(runtime_dir(&root), root.join(".runtime"));
    }

    #[test]
    fn uv_paths_are_rooted() {
        let root = PathBuf::from(r"C:\Apps\MyApp");
        let (uv, uvx, uvw) = uv_paths(&root);
        assert_eq!(uv, root.join("uv.exe"));
        assert_eq!(uvx, root.join("uvx.exe"));
        assert_eq!(uvw, root.join("uvw.exe"));
    }
}
