use crate::config::get_config;
use crate::selection::get_selection_text;
use crate::StringWrapper;
use crate::APP;
use tauri::{AppHandle, Manager, Window};
use toml::Value;
#[cfg(any(target_os = "macos", target_os = "windows"))]
use window_shadows::set_shadow;

pub fn build_translate_window(
    label: &str,
    title: &str,
    handle: &AppHandle,
) -> Result<Window, String> {
    let (width, height) = get_window_size();
    // 对于Mac来说这里获取直接可用的逻辑坐标
    // 对于Windows和Linux，这里仅获取物理坐标，用于确保窗口创建在指定的显示器上
    // 获取到真实的显示器信息之后再做转换
    let (x, y) = get_mouse_location().unwrap();
    let builder =
        tauri::WindowBuilder::new(handle, label, tauri::WindowUrl::App("index.html".into()))
            .inner_size(width, height)
            .focused(true)
            .visible(false)
            .title(title);

    #[cfg(target_os = "macos")]
    {
        let builder = builder
            .title_bar_style(tauri::TitleBarStyle::Overlay)
            .hidden_title(true);
        let window = match label {
            "persistent" => builder.center().skip_taskbar(false).build().unwrap(),
            _ => builder.position(x, y).skip_taskbar(true).build().unwrap(),
        };
        // 获取窗口所在的显示器信息
        let _monitor = window.current_monitor().unwrap().unwrap();
        // 获取到显示器信息之后再移动窗口，确保窗口大小正确
        window
            .set_position(tauri::LogicalPosition::new(x, y))
            .unwrap();
        set_shadow(&window, true).unwrap_or_default();
        Ok(window)
    }

    #[cfg(target_os = "windows")]
    {
        let builder = builder.decorations(false);
        let window = match label {
            "persistent" => builder.skip_taskbar(false).build().unwrap(),
            _ => builder.skip_taskbar(true).build().unwrap(),
        };
        // 移动窗口到鼠标所在显示器上
        window
            .set_position(tauri::PhysicalPosition::new(x, y))
            .unwrap();
        // 获取窗口所在的显示器信息
        let monitor = window.current_monitor().unwrap().unwrap();
        match label {
            "persistent" => window.center().unwrap(),
            _ => {
                // 用显示器信息将物理坐标做转换
                let (x, y) = convert_mouse_location((x, y), monitor).unwrap();
                // 获取到显示器信息之后再移动窗口，确保窗口大小正确
                window
                    .set_position(tauri::LogicalPosition::new(x, y))
                    .unwrap();
            }
        }
        set_shadow(&window, true).unwrap_or_default();
        Ok(window)
    }

    #[cfg(target_os = "linux")]
    {
        let builder = builder.transparent(true).decorations(false);
        let window = match label {
            "persistent" => builder.skip_taskbar(false).build().unwrap(),
            _ => builder.skip_taskbar(true).build().unwrap(),
        };
        // 移动窗口到鼠标所在显示器上
        window
            .set_position(tauri::PhysicalPosition::new(x, y))
            .unwrap();
        // 获取窗口所在的显示器信息
        let monitor = window.current_monitor().unwrap().unwrap();
        match label {
            "persistent" => window.center().unwrap(),
            _ => {
                // 用显示器信息将物理坐标做转换
                let (x, y) = convert_mouse_location((x, y), monitor).unwrap();
                // 获取到显示器信息之后再移动窗口，确保窗口大小正确
                window
                    .set_position(tauri::LogicalPosition::new(x, y))
                    .unwrap();
            }
        }
        Ok(window)
    }
}

pub fn build_ocr_window(handle: &AppHandle) -> Result<Window, String> {
    let window =
        tauri::WindowBuilder::new(handle, "ocr", tauri::WindowUrl::App("index.html".into()))
            .inner_size(800.0, 400.0)
            .min_inner_size(600.0, 400.0)
            .center()
            .focused(true)
            .title("OCR")
            .build()
            .unwrap();
    Ok(window)
}

// 获取默认窗口大小
fn get_window_size() -> (f64, f64) {
    let width: f64 = get_config("window_width", Value::from(400), APP.get().unwrap().state())
        .as_integer()
        .unwrap() as f64;
    let height: f64 = get_config(
        "window_height",
        Value::from(500),
        APP.get().unwrap().state(),
    )
    .as_integer()
    .unwrap() as f64;
    (width, height)
}

#[cfg(any(target_os = "linux", target_os = "windows"))]
fn convert_mouse_location(
    location: (f64, f64),
    monitor: tauri::Monitor,
) -> Result<(f64, f64), String> {
    let (mut x, mut y) = location;
    let (width, height) = get_window_size();
    let monitor_size = monitor.size();
    let dpi = monitor.scale_factor();
    x /= dpi;
    y /= dpi;
    if x + width > monitor_size.width as f64 / dpi {
        x -= width;
        if x < 0.0 {
            x = 0.0;
        }
    }
    if y + height > monitor_size.height as f64 / dpi {
        y -= height;
        if y < 0.0 {
            y = 0.0;
        }
    }

    Ok((x, y))
}
// 获取鼠标物理坐标
#[cfg(target_os = "linux")]
fn get_mouse_location() -> Result<(f64, f64), String> {
    use mouse_position::mouse_position::Mouse;

    let position = Mouse::get_mouse_position();
    if let Mouse::Position { x: pos_x, y: pos_y } = position {
        Ok((pos_x as f64, pos_y as f64))
    } else {
        Err("get cursorpos error".to_string())
    }
}
// 获取鼠标物理坐标
#[cfg(target_os = "windows")]
fn get_mouse_location() -> Result<(f64, f64), String> {
    use windows::Win32::Foundation::POINT;
    use windows::Win32::UI::WindowsAndMessaging::GetCursorPos;

    let mut point = POINT { x: 0, y: 0 };

    unsafe {
        if GetCursorPos(&mut point).as_bool() {
            Ok((point.x as f64, point.y as f64))
        } else {
            Err("get cursorpos error".to_string())
        }
    }
}
// 获取鼠标逻辑坐标
#[cfg(target_os = "macos")]
fn get_mouse_location() -> Result<(f64, f64), String> {
    use core_graphics::display::CGDisplay;
    use core_graphics::event::CGEvent;
    use core_graphics::event_source::{CGEventSource, CGEventSourceStateID};
    let display = CGDisplay::main();
    let mode = display.display_mode().unwrap();
    let event =
        CGEvent::new(CGEventSource::new(CGEventSourceStateID::CombinedSessionState).unwrap());
    let point = event.unwrap().location();
    let mut x = point.x;
    let mut y = point.y;
    let (width, height) = get_window_size();
    if x + width > mode.width() as f64 {
        x = x - width;
        if x < 0.0 {
            x = 0.0;
        }
    }
    if y + height > mode.height() as f64 {
        y = y - height;
        if y < 0.0 {
            y = 0.0;
        }
    }
    return Ok((x, y));
}

// 划词翻译
pub fn translate_window() {
    // 获取选择文本
    let mut text = String::new();
    if let Ok(v) = get_selection_text() {
        text = v;
    }
    let handle = APP.get().unwrap();
    // 写入状态备用
    let state: tauri::State<StringWrapper> = handle.state();
    state.0.lock().unwrap().replace_range(.., &text);
    // 创建窗口
    match handle.get_window("translator") {
        Some(window) => {
            window.set_focus().unwrap();
        }
        None => {
            let _window = build_translate_window("translator", "Translator", handle).unwrap();
        }
    };
}

// 持久窗口
pub fn persistent_window() {
    let handle = APP.get().unwrap();
    match handle.get_window("persistent") {
        Some(window) => {
            window.set_focus().unwrap();
        }
        None => {
            let _window = build_translate_window("persistent", "Persistent", handle).unwrap();
        }
    };
}

// popclip划词翻译
pub fn popclip_window(text: String) {
    let handle = APP.get().unwrap();

    let state: tauri::State<StringWrapper> = handle.state();
    state.0.lock().unwrap().replace_range(.., &text);

    match handle.get_window("popclip") {
        Some(window) => {
            window.set_focus().unwrap();
        }
        None => {
            let _window = build_translate_window("popclip", "PopClip", handle).unwrap();
        }
    };
}

// OCR
#[allow(dead_code)]
pub fn ocr_window() {
    let handle = APP.get().unwrap();

    // 读取剪切板图片

    match handle.get_window("ocr") {
        Some(window) => {
            window.close().unwrap();
        }
        None => {
            let _main_window = build_ocr_window(handle).unwrap();
        }
    };
}
