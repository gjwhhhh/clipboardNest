// 窗口管理模块
use tauri::WebviewWindow;

/// 计算窗口居中位置
pub fn calculate_center_position(
    screen_width: i32,
    screen_height: i32,
    window_width: i32,
    window_height: i32,
) -> (i32, i32) {
    let x = (screen_width - window_width) / 2;
    let y = (screen_height - window_height) / 2;
    (x.max(0), y.max(0))
}

pub fn position_near_mouse(window: &WebviewWindow) -> Result<(), Box<dyn std::error::Error>> {
    let window_width = 400;
    let window_height = 500;

    // 使用 Tauri API 获取真实屏幕尺寸
    if let Ok(Some(monitor)) = window.current_monitor() {
        let size = monitor.size();
        let scale = monitor.scale_factor();
        let screen_w = (size.width as f64 / scale) as i32;
        let screen_h = (size.height as f64 / scale) as i32;

        let (x, y) = calculate_center_position(screen_w, screen_h, window_width, window_height);
        window.set_position(tauri::Position::Logical(tauri::LogicalPosition {
            x: x as f64,
            y: y as f64,
        }))?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_计算居中位置() {
        let (x, y) = calculate_center_position(1920, 1080, 400, 500);
        assert_eq!(x, 760);
        assert_eq!(y, 290);
    }

    #[test]
    fn test_小屏幕也能居中() {
        let (x, y) = calculate_center_position(800, 600, 400, 500);
        assert_eq!(x, 200);
        assert_eq!(y, 50);
    }

    #[test]
    fn test_窗口大于屏幕时返回0() {
        let (x, y) = calculate_center_position(300, 200, 400, 500);
        assert_eq!(x, 0);
        assert_eq!(y, 0);
    }
}
