#[path = "../src/logging.rs"]
mod logging;

#[test]
fn logging_init_creates_file() {
    let tmp = tempfile::tempdir().unwrap();
    let log_path = logging::init(tmp.path()).expect("logging init should succeed");
    assert!(log_path.exists());
}
