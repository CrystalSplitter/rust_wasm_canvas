use wasm_bindgen::prelude::*;

use crate::inputs;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen]
    #[derive(Debug, Clone)]
    pub type InputBinding;

    #[wasm_bindgen(method, js_name = "getMouseX")]
    pub fn get_mouse_x_(this: &InputBinding) -> f32;

    #[wasm_bindgen(method, js_name = "getMouseY")]
    pub fn get_mouse_y_(this: &InputBinding) -> f32;
}

impl inputs::Input for InputBinding {
    fn get_mouse_x(&self) -> f32 {
        self.get_mouse_x_()
    }

    fn get_mouse_y(&self) -> f32 {
        self.get_mouse_y_()
    }

    fn get_mouse_view_x(&self) -> f32 {
        panic!()
    }

    fn get_mouse_view_y(&self) -> f32 {
        panic!()
    }
}

/// Print a log to the JS Console.
pub fn log(s: &str) {
    web_sys::console::log_1(&s.into());
}

/// Print an error to the JS Console.
pub fn error(s: &str) {
    web_sys::console::error_1(&s.into());
}

/// Print an error to the JS Console.
pub fn warn(s: &str) {
    web_sys::console::warn_1(&s.into());
}

pub fn millis_now() -> u32 {
    js_sys::Date::new_0().get_utc_milliseconds()
}
