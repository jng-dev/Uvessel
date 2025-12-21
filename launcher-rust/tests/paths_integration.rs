#[path = "../src/paths.rs"]
mod paths;

#[test]
fn uv_paths_use_root() {
    let root = std::path::PathBuf::from(r"C:\Apps\MyApp");
    let (uv, uvx, uvw) = paths::uv_paths(&root);
    assert_eq!(uv, root.join("uv.exe"));
    assert_eq!(uvx, root.join("uvx.exe"));
    assert_eq!(uvw, root.join("uvw.exe"));
}
