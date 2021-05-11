use std::collections::*;

use serde_wasm_bindgen;
use wasm_bindgen::prelude::*;
use web_sys;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen]
    #[derive(Debug, Clone)]
    pub type InputBinding;

    #[wasm_bindgen(method, js_name = "getMouseX")]
    pub fn get_mouse_x(this: &InputBinding) -> f32;

    #[wasm_bindgen(method, js_name = "getMouseY")]
    pub fn get_mouse_y(this: &InputBinding) -> f32;
}
