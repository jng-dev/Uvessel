#[path = "../src/installer.rs"]
mod installer;
#[path = "../src/payload.rs"]
mod payload;
#[path = "../src/fs_ops.rs"]
mod fs_ops;
#[path = "../src/state.rs"]
mod state;
#[path = "../src/shortcuts.rs"]
mod shortcuts;
#[path = "../src/paths.rs"]
mod paths;
#[path = "../src/uv.rs"]
mod uv;

use std::fs;

#[test]
fn installer_writes_state_and_runs_uv_commands() {
    let tmp = tempfile::tempdir().unwrap();
    let src_root = tmp.path().join("src");
    let install_root = tmp.path().join("install");
    fs::create_dir_all(&src_root).unwrap();

    let self_exe = src_root.join("MyApp.exe");
    fs::write(&self_exe, "binary").unwrap();

    let ensure_uv = |root: &std::path::Path| -> anyhow::Result<()> {
        fs::write(root.join("uv.exe"), "fake")?;
        Ok(())
    };

    let mut seen = Vec::new();
    let exec = |cmd: &mut std::process::Command| -> anyhow::Result<std::process::ExitStatus> {
        let program = cmd.get_program().to_string_lossy().to_string();
        let args: Vec<String> = cmd.get_args().map(|a| a.to_string_lossy().to_string()).collect();
        seen.push((program, args));
        #[cfg(windows)]
        {
            use std::os::windows::process::ExitStatusExt;
            Ok(std::process::ExitStatus::from_raw(0))
        }
        #[cfg(not(windows))]
        {
            Err(anyhow::anyhow!("windows-only test"))
        }
    };

    let create_shortcut =
        |start_menu: &std::path::Path, name: &str, target: &std::path::Path, _icon: Option<&std::path::Path>| {
        let out = start_menu.join(format!("{name}.lnk"));
        fs::create_dir_all(start_menu)?;
        fs::write(&out, target.display().to_string())?;
        Ok(out)
    };

    let launch = |_exe: &std::path::Path| -> anyhow::Result<()> { Ok(()) };

    installer::run_with_deps(
        &src_root,
        &install_root,
        "MyApp",
        &self_exe,
        ensure_uv,
        exec,
        create_shortcut,
        launch,
    )
    .unwrap();

    let state_path = state::state_path(&install_root);
    assert!(state_path.exists());

    let st = state::read_state(&state_path).unwrap();
    let expected_proj = find_project_rel(&install_root).unwrap();
    assert_eq!(st.project_rel, expected_proj);
    assert!(install_root.join("MyApp.exe").exists());

    assert_eq!(seen.len(), 2);
    assert!(seen[0].1.starts_with(&["python".to_string(), "install".to_string()]));
    assert!(seen[1].1.starts_with(&["sync".to_string()]));
}

fn find_project_rel(root: &std::path::Path) -> anyhow::Result<String> {
    let app_dir = root.join("app");
    for entry in fs::read_dir(&app_dir)? {
        let entry = entry?;
        if !entry.file_type()?.is_dir() {
            continue;
        }
        let candidate = entry.path();
        if candidate.join("pyproject.toml").exists() {
            let rel = candidate.strip_prefix(root).unwrap_or(&candidate);
            return Ok(rel.to_string_lossy().to_string());
        }
    }
    Err(anyhow::anyhow!(
        "no project found under {}",
        app_dir.display()
    ))
}
