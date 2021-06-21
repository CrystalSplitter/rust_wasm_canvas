use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{HtmlCanvasElement, HtmlElement, MouseEvent};

pub trait Input {
    fn get_mouse_x(&self) -> f32;
    fn get_mouse_y(&self) -> f32;

    /// Viewport X position, generally ranging from -1 to 1 (with 0
    /// being the centre). Can exceed these bounds if the mouse
    /// lies outside the canvas bounds.
    fn get_mouse_view_x(&self) -> f32;

    /// Viewport Y position, generally ranging from -1 to 1 (with 0
    /// being the centre). Can exceed these bounds if the mouse
    /// lies outside the canvas bounds.
    fn get_mouse_view_y(&self) -> f32;
}

#[derive(Debug, Clone)]
pub struct InputBinding {
    pub mouse_x: f32,
    pub mouse_y: f32,
    pub canvas_width: u32,
    pub canvas_height: u32,
}

impl Input for InputBinding {
    fn get_mouse_x(&self) -> f32 {
        self.mouse_x as f32
    }

    fn get_mouse_y(&self) -> f32 {
        self.mouse_y as f32
    }

    fn get_mouse_view_x(&self) -> f32 {
        let canvas_width = self.canvas_width as f32;
        self.mouse_x / canvas_width - 0.5 * canvas_width
    }

    fn get_mouse_view_y(&self) -> f32 {
        let canvas_height = self.canvas_height as f32;
        self.mouse_y / canvas_height - 0.5 * canvas_height
    }
}

/// Add a listener to a type coercible to an HtmlElement.
///
/// # Arguments
///
/// * `elem` - HtmlElement to add the listener too.
/// * `dom_event` - DOM event string to trigger the callback.
/// * `cb` - A Box around the callback to trigger on the event.
pub fn add_listener<CbT>(elem: &HtmlElement, dom_event: &str, cb: Box<CbT>) -> Result<(), JsValue>
where
    CbT: FnMut(web_sys::Event) + 'static,
{
    let closure = Closure::wrap(cb as Box<dyn FnMut(_)>);
    elem.add_event_listener_with_callback(dom_event, closure.as_ref().unchecked_ref())?;
    closure.forget();
    Ok(())
}

/// Get the mouse position from a given MouseEvent.
pub fn get_mouse_pos(canvas: &HtmlCanvasElement, evt: &MouseEvent) -> Result<InputBinding, String> {
    let rect = canvas.get_bounding_client_rect();
    let scale_x: f32 = canvas.width() as f32 / rect.width() as f32;
    let scale_y: f32 = canvas.height() as f32 / rect.height() as f32;
    Ok(InputBinding {
        mouse_x: ((evt.client_x() as f32 - rect.left() as f32) * scale_x),
        mouse_y: ((evt.client_y() as f32 - rect.top() as f32) * scale_y),
        canvas_width: canvas.width(),
        canvas_height: canvas.height(),
    })
}
