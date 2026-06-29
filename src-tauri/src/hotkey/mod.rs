// 全局快捷键模块
use std::sync::Mutex;
use tauri::{AppHandle, Manager, WebviewWindow};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut};

pub fn default_hotkey() -> &'static str {
    if cfg!(target_os = "macos") {
        "Cmd+Shift+V"
    } else {
        "Ctrl+Shift+V"
    }
}

pub struct HotkeyManager {
    current_shortcut: Mutex<String>,
}

impl HotkeyManager {
    pub fn new(initial: &str) -> Self {
        Self {
            current_shortcut: Mutex::new(initial.to_string()),
        }
    }

    pub fn current(&self) -> String {
        self.current_shortcut.lock().unwrap().clone()
    }

    pub fn update(&self, new_shortcut: &str) {
        *self.current_shortcut.lock().unwrap() = new_shortcut.to_string();
    }
}

pub fn register_hotkey(
    app: &AppHandle,
    hotkey_str: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let shortcut = parse_hotkey(hotkey_str)?;
    let window = app.get_webview_window("main").unwrap();

    app.global_shortcut()
        .on_shortcut(shortcut, move |_app, _shortcut, event| {
            if event.state == tauri_plugin_global_shortcut::ShortcutState::Pressed {
                toggle_window(&window);
            }
        })?;

    Ok(())
}

/// 取消注册旧快捷键并注册新快捷键
pub fn reregister_hotkey(
    app: &AppHandle,
    old_hotkey: &str,
    new_hotkey: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    match parse_hotkey(old_hotkey) {
        Ok(old_shortcut) => {
            if let Err(err) = app.global_shortcut().unregister(old_shortcut) {
                log::debug!("取消注册旧快捷键失败，继续注册新快捷键: {}", err);
            }
        }
        Err(err) => {
            log::debug!("旧快捷键格式无效，跳过取消注册: {}", err);
        }
    }
    register_hotkey(app, new_hotkey)?;
    Ok(())
}

pub(crate) fn parse_hotkey(hotkey_str: &str) -> Result<Shortcut, Box<dyn std::error::Error>> {
    let parts: Vec<&str> = hotkey_str.split('+').collect();
    let mut modifiers = Modifiers::empty();
    let mut code = None;

    for part in &parts {
        match part.trim() {
            "Cmd" | "Command" => modifiers |= Modifiers::META,
            "Shift" => modifiers |= Modifiers::SHIFT,
            "Ctrl" | "Control" => modifiers |= Modifiers::CONTROL,
            "Alt" | "Option" => modifiers |= Modifiers::ALT,
            key => {
                code = Some(match key {
                    // 字母键 A-Z
                    "A" => Code::KeyA,
                    "B" => Code::KeyB,
                    "C" => Code::KeyC,
                    "D" => Code::KeyD,
                    "E" => Code::KeyE,
                    "F" => Code::KeyF,
                    "G" => Code::KeyG,
                    "H" => Code::KeyH,
                    "I" => Code::KeyI,
                    "J" => Code::KeyJ,
                    "K" => Code::KeyK,
                    "L" => Code::KeyL,
                    "M" => Code::KeyM,
                    "N" => Code::KeyN,
                    "O" => Code::KeyO,
                    "P" => Code::KeyP,
                    "Q" => Code::KeyQ,
                    "R" => Code::KeyR,
                    "S" => Code::KeyS,
                    "T" => Code::KeyT,
                    "U" => Code::KeyU,
                    "V" => Code::KeyV,
                    "W" => Code::KeyW,
                    "X" => Code::KeyX,
                    "Y" => Code::KeyY,
                    "Z" => Code::KeyZ,
                    // 数字键 0-9
                    "0" => Code::Digit0,
                    "1" => Code::Digit1,
                    "2" => Code::Digit2,
                    "3" => Code::Digit3,
                    "4" => Code::Digit4,
                    "5" => Code::Digit5,
                    "6" => Code::Digit6,
                    "7" => Code::Digit7,
                    "8" => Code::Digit8,
                    "9" => Code::Digit9,
                    // 功能键 F1-F12
                    "F1" => Code::F1,
                    "F2" => Code::F2,
                    "F3" => Code::F3,
                    "F4" => Code::F4,
                    "F5" => Code::F5,
                    "F6" => Code::F6,
                    "F7" => Code::F7,
                    "F8" => Code::F8,
                    "F9" => Code::F9,
                    "F10" => Code::F10,
                    "F11" => Code::F11,
                    "F12" => Code::F12,
                    // 特殊键
                    "Space" => Code::Space,
                    "Enter" => Code::Enter,
                    "Tab" => Code::Tab,
                    "Backspace" => Code::Backspace,
                    "Delete" => Code::Delete,
                    "Escape" => Code::Escape,
                    _ => return Err(format!("未知按键: {}", key).into()),
                });
            }
        }
    }

    let code = code.ok_or("未指定按键")?;
    Ok(Shortcut::new(Some(modifiers), code))
}

pub fn toggle_window(window: &WebviewWindow) {
    if window.is_visible().unwrap_or(false) {
        let _ = window.hide();
    } else {
        let _ = crate::window::position_near_mouse(window);
        crate::window::show_window(window);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_默认快捷键按平台返回() {
        if cfg!(target_os = "macos") {
            assert_eq!(default_hotkey(), "Cmd+Shift+V");
        } else {
            assert_eq!(default_hotkey(), "Ctrl+Shift+V");
        }
    }

    #[test]
    fn test_可以解析_windows_默认快捷键() {
        assert!(parse_hotkey("Ctrl+Shift+V").is_ok());
    }
}
