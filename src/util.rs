use std::cell::RefCell;
use std::rc::{Rc, Weak};

pub type Rfc<T> = Rc<RefCell<T>>;
pub type Wfc<T> = Weak<RefCell<T>>;

/// Return the aspect ratio of a canvas.
pub fn canvas_aspect_ratio(canvas: &web_sys::HtmlCanvasElement) -> f32 {
    canvas.client_width() as f32 / canvas.client_height() as f32
}

#[macro_export]
macro_rules! build_setter {
    ($name:ident, $arg_ty:ty) => {
        pub fn $name(self, $name: $arg_ty) -> Self {
            Self {
                $name: Some($name),
                ..self
            }
        }
    };
}

#[macro_export]
macro_rules! build_setter_defaulted {
    ($name:ident, $arg_ty:ty) => {
        pub fn $name(self, $name: $arg_ty) -> Self {
            Self {
                $name: $name,
                ..self
            }
        }
    };
}
