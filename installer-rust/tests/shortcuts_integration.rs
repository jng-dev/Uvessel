#[path = "../src/shortcuts.rs"]
mod shortcuts;

use std::fs;

#[test]
fn create_shortcut_writes_lnk_file() {
    let tmp = tempfile::tempdir().unwrap();
    let start_menu_dir = tmp.path().join("Programs");
    let target = tmp.path().join("app.exe");
    fs::write(&target, "binary").unwrap();

    let lnk = shortcuts::create_start_menu_shortcut(
        &start_menu_dir,
        "MyApp",
        &target,
        None,
    )
    .unwrap();

    assert!(lnk.exists());
}
