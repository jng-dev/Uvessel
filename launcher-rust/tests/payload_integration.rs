#[path = "../src/fs_ops.rs"]
mod fs_ops;
#[path = "../src/payload.rs"]
mod payload;

#[test]
fn install_payload_extracts_embedded_zip() {
    let tmp = tempfile::tempdir().unwrap();
    payload::install_payload(tmp.path()).unwrap();
    let placeholder = tmp
        .path()
        .join("app")
        .join("put-project-here.txt");
    assert!(placeholder.exists());
}
