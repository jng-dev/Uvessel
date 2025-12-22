#[path = "../src/fs_ops.rs"]
mod fs_ops;

use std::fs;

#[test]
fn copy_and_move_work_in_temp_dir() {
    let tmp = tempfile::tempdir().unwrap();
    let src = tmp.path().join("src.txt");
    let copy = tmp.path().join("copy.txt");
    let moved = tmp.path().join("moved.txt");

    fs::write(&src, "data").unwrap();
    fs_ops::copy_file_with_retry(&src, &copy, 3).unwrap();
    assert!(copy.exists());

    fs_ops::move_file_with_retry(&copy, &moved, 3).unwrap();
    assert!(!copy.exists());
    assert_eq!(fs::read_to_string(&moved).unwrap(), "data");
}
