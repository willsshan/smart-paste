use tauri::{AppHandle, Emitter, Manager, PhysicalPosition, Position, Runtime, WebviewWindow};

pub fn position_main_window<R: Runtime>(app: &AppHandle<R>) {
    let Some(window) = app.get_webview_window("main") else {
        return;
    };

    let Ok(cursor) = app.cursor_position() else {
        return;
    };

    let Ok(monitor) = window.current_monitor() else {
        return;
    };

    let Some(monitor) = monitor else {
        return;
    };

    let monitor_pos = monitor.position();
    let monitor_size = monitor.size();
    let window_size = window.outer_size().ok();
    let width = window_size.map(|size| size.width as i32).unwrap_or(360);
    let height = window_size.map(|size| size.height as i32).unwrap_or(420);

    let desired_x = cursor.x as i32 + 18;
    let desired_y = cursor.y as i32 + 18;

    let min_x = monitor_pos.x;
    let min_y = monitor_pos.y;
    let max_x = monitor_pos.x + monitor_size.width as i32 - width - 8;
    let max_y = monitor_pos.y + monitor_size.height as i32 - height - 8;

    let x = desired_x.clamp(min_x + 8, max_x.max(min_x + 8));
    let y = desired_y.clamp(min_y + 8, max_y.max(min_y + 8));

    let _ = window.set_position(Position::Physical(PhysicalPosition::new(x, y)));
}

pub fn show_main_window<R: Runtime>(app: &AppHandle<R>) -> Result<(), String> {
    let Some(window) = app.get_webview_window("main") else {
        return Err("main window not found".to_string());
    };

    position_main_window(app);
    window.show().map_err(|error| error.to_string())?;
    let _ = window.set_focus();
    let _ = window.emit("overlay-opened", ());

    Ok(())
}

#[tauri::command]
pub fn hide_window(window: WebviewWindow) -> Result<(), String> {
    window.hide().map_err(|error| error.to_string())
}
