#[path = "../src/runner.rs"]
mod runner;
#[path = "../src/state.rs"]
mod state;

use std::{fs, path::PathBuf};

#[test]
fn runner_executes_bootstrap_and_run() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    fs::create_dir_all(root.join(".runtime")).unwrap();

    let proj = root.join("app").join("proj");
    fs::create_dir_all(&proj).unwrap();
    fs::write(proj.join("pyproject.toml"), "[project]\nname='x'\n").unwrap();
    fs::write(proj.join("main.py"), "print('hi')").unwrap();
    fs::write(proj.join("uv.lock"), "lock").unwrap();

    fs::write(root.join("uv.exe"), "fake").unwrap();

    let st = state::State {
        project_rel: PathBuf::from("app").join("proj").to_string_lossy().to_string(),
        entry: state::EntryPoint::PythonFile("main.py".to_string()),
        lock_mtime_unix: 0,
        installed: true,
    };
    state::write_state(&state::state_path(root), &st).unwrap();

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

    runner::run_with_executor(root, exec).unwrap();

    assert_eq!(seen.len(), 3);
    assert!(seen[0].1.starts_with(&["python".to_string(), "install".to_string()]));
    assert!(seen[1].1.starts_with(&["sync".to_string()]));
    assert!(seen[2].1.starts_with(&["run".to_string(), "python".to_string(), "main.py".to_string()]));
}
