fn main() {
    const COMMANDS: &[&str] = &[
        "start_monitoring",
        "stop_monitoring",
        "get_text",
        "set_text",
        "get_image",
        "set_image",
        "update_clipboard_content",
    ];

    tauri_plugin::Builder::new(COMMANDS)
        .android_path("android")
        .ios_path("ios")
        .build();
}
