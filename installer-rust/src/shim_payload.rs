pub const EMBEDDED_SHIM: &[u8] =
    include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/embedded/launcher.exe"));
