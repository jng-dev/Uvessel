pub const EMBEDDED_UPDATER: &[u8] =
    include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/embedded/updater.exe"));
