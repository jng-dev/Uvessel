#[path = "../src/fs_ops.rs"]
mod fs_ops;
#[path = "../src/payload.rs"]
mod payload;
#[path = "../src/shortcuts.rs"]
mod shortcuts;
#[path = "../src/state.rs"]
mod state;
#[path = "../src/uv.rs"]
mod uv;
#[path = "../src/paths.rs"]
mod paths;
#[path = "../src/main.rs"]
mod main_mod;

use std::fs;

#[test]
fn select_mode_switches_on_state_file() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();

    assert_eq!(main_mod::select_mode(root), main_mod::Mode::Installer);

    let runtime = root.join(".runtime");
    fs::create_dir_all(&runtime).unwrap();
    let state_path = runtime.join("state.json");
    fs::write(
        &state_path,
        r#"{"project_rel":"app/proj","entry":{"kind":"python_file","value":"main.py"},"lock_mtime_unix":0,"installed":true}"#,
    )
    .unwrap();

    assert_eq!(main_mod::select_mode(root), main_mod::Mode::Runner);
}
