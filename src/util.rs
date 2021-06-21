use std::rc::{Rc,Weak};
use std::cell::RefCell;

pub type Rfc<T> = Rc<RefCell<T>>;
pub type Wfc<T> = Weak<RefCell<T>>;

/// Return the aspect ratio of a canvas.
pub fn canvas_aspect_ratio(canvas: &web_sys::HtmlCanvasElement) -> f32 {
    canvas.client_width() as f32 / canvas.client_height() as f32
}
