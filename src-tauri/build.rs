fn main() {
    // Skip icon processing for now
    std::env::set_var("TAURI_SKIP_ICON_CHECK", "1");

    tauri_build::build()
}