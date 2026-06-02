pub fn native_target_triple() -> String {
    #[cfg(target_os = "windows")]
    { "x86_64-pc-windows-msvc".to_string() }
    #[cfg(target_os = "linux")]
    { "x86_64-unknown-linux-gnu".to_string() }
    #[cfg(target_os = "macos")]
    { "x86_64-apple-darwin".to_string() }
    #[cfg(target_arch = "aarch64")]
    { "aarch64-unknown-linux-gnu".to_string() }
}

pub fn target_triple_for(platform: &str) -> String {
    match platform {
        "windows" => "x86_64-pc-windows-msvc".to_string(),
        "linux" => "x86_64-unknown-linux-gnu".to_string(),
        "macos" => "x86_64-apple-darwin".to_string(),
        "android" => "aarch64-linux-android".to_string(),
        "ios" => "aarch64-apple-ios".to_string(),
        "wasm" => "wasm32-unknown-unknown".to_string(),
        _ => native_target_triple(),
    }
}
