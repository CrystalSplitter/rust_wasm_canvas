use web_sys;

pub struct InputBinding {
    pub mouse_x: i32,
    pub mouse_y: i32,
}

pub fn get_mouse_pos(
    canvas: &web_sys::HtmlCanvasElement,
    evt: web_sys::MouseEvent,
) -> Result<InputBinding, String> {
    let rect = canvas.get_bounding_client_rect();
    let scale_x: f32 = canvas.width() as f32 / rect.width() as f32;
    let scale_y: f32 = canvas.height() as f32 / rect.height() as f32;
    Ok(InputBinding {
        mouse_x: ((evt.client_x() as f32 - rect.left() as f32) * scale_x) as i32,
        mouse_y: ((evt.client_y() as f32 - rect.top() as f32) * scale_y) as i32,
    })
}
