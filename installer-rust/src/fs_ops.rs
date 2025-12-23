use anyhow::{Context, Result};
use std::{
    fs,
    path::{Path, PathBuf},
    time::Duration,
};

fn retry<F>(mut op: F, attempts: usize) -> Result<()>
where
    F: FnMut() -> Result<()>,
{
    let mut delay = Duration::from_millis(200);
    for i in 0..attempts {
        match op() {
            Ok(()) => return Ok(()),
            Err(err) => {
                if i + 1 == attempts {
                    return Err(err);
                }
            }
        }
        std::thread::sleep(delay);
        delay = std::cmp::min(delay * 2, Duration::from_secs(2));
    }
    Ok(())
}

fn temp_path_for(dest: &Path) -> Result<PathBuf> {
    let parent = dest.parent().context("dest has no parent")?;
    let name = dest
        .file_name()
        .context("dest has no filename")?
        .to_string_lossy();
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or(Duration::from_millis(0))
        .as_nanos();
    Ok(parent.join(format!("{name}.tmp-{nonce}")))
}

#[cfg(test)]
fn copy_file_atomic(src: &Path, dest: &Path) -> Result<()> {
    let tmp = temp_path_for(dest)?;
    fs::copy(src, &tmp)
        .with_context(|| format!("copy {} -> {}", src.display(), tmp.display()))?;
    if dest.exists() {
        fs::remove_file(dest)
            .with_context(|| format!("remove {}", dest.display()))?;
    }
    fs::rename(&tmp, dest)
        .with_context(|| format!("rename {} -> {}", tmp.display(), dest.display()))?;
    Ok(())
}

#[cfg(test)]
fn move_file_atomic(src: &Path, dest: &Path) -> Result<()> {
    if dest.exists() {
        fs::remove_file(dest)
            .with_context(|| format!("remove {}", dest.display()))?;
    }
    fs::rename(src, dest)
        .with_context(|| format!("rename {} -> {}", src.display(), dest.display()))?;
    Ok(())
}

fn write_bytes_atomic(dest: &Path, bytes: &[u8]) -> Result<()> {
    let tmp = temp_path_for(dest)?;
    fs::write(&tmp, bytes).with_context(|| format!("write {}", tmp.display()))?;
    if dest.exists() {
        fs::remove_file(dest)
            .with_context(|| format!("remove {}", dest.display()))?;
    }
    fs::rename(&tmp, dest)
        .with_context(|| format!("rename {} -> {}", tmp.display(), dest.display()))?;
    Ok(())
}

#[cfg(test)]
pub fn copy_file_with_retry(src: &Path, dest: &Path, attempts: usize) -> Result<()> {
    retry(|| copy_file_atomic(src, dest), attempts)
}

#[cfg(test)]
pub fn move_file_with_retry(src: &Path, dest: &Path, attempts: usize) -> Result<()> {
    retry(|| move_file_atomic(src, dest), attempts)
}

pub fn write_bytes_with_retry(dest: &Path, bytes: &[u8], attempts: usize) -> Result<()> {
    retry(|| write_bytes_atomic(dest, bytes), attempts)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn copy_file_with_retry_copies_contents() {
        let tmp = tempfile::tempdir().unwrap();
        let src = tmp.path().join("src.txt");
        let dest = tmp.path().join("dest.txt");
        let mut f = fs::File::create(&src).unwrap();
        writeln!(f, "hello").unwrap();

        copy_file_with_retry(&src, &dest, 3).unwrap();

        let out = fs::read_to_string(&dest).unwrap();
        assert!(out.contains("hello"));
    }

    #[test]
    fn move_file_with_retry_moves_contents() {
        let tmp = tempfile::tempdir().unwrap();
        let src = tmp.path().join("src.txt");
        let dest = tmp.path().join("dest.txt");
        fs::write(&src, "moved").unwrap();

        move_file_with_retry(&src, &dest, 3).unwrap();

        assert!(!src.exists());
        let out = fs::read_to_string(&dest).unwrap();
        assert_eq!(out, "moved");
    }
}
