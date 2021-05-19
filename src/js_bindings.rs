use std::collections::*;

use serde_wasm_bindgen;
use wasm_bindgen::prelude::*;
use web_sys;

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
}
