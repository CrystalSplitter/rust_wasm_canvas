use web_sys;

pub fn set_panic_hook() {
    // When the `console_error_panic_hook` feature is enabled, we can call the
    // `set_panic_hook` function at least once during initialization, and then
    // we will get better error messages if our code ever panics.
    //
    // For more details see
    // https://github.com/rustwasm/console_error_panic_hook#readme
    //#[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

pub fn get_mouse_pos(canvas: web_sys::HtmlCanvasElement, evt: web_sys::MouseEvent) -> (i32, i32) {
    let rect = canvas.get_bounding_client_rect();
    let scale_x: f32 = canvas.width() as f32 / rect.width() as f32;
    let scale_y: f32 = canvas.height() as f32 / rect.height() as f32;
    (
        ((evt.client_x() as f32 - rect.left() as f32) * scale_x) as i32,
        ((evt.client_y() as f32 - rect.top() as f32) * scale_y) as i32,
    )
}
