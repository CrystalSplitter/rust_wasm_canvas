extern crate console_error_panic_hook;
extern crate nalgebra as na;
extern crate serde_wasm_bindgen;
extern crate wasm_bindgen;
extern crate web_sys;

use wasm_bindgen::prelude::*;
use web_sys::*;

mod geometry;
mod inputs;
mod js_bindings;
mod rendering;
mod spin;
mod transform;
mod util;

#[wasm_bindgen]
pub fn get_transform_mat(x: f32, y: f32) -> Vec<f32> {
    let v: na::Matrix3<f32> = na::Matrix3::identity().append_translation(&na::Vector2::new(x, y));
    v.as_slice().to_vec()
}

#[wasm_bindgen]
pub fn bind_game(
    context: &web_sys::WebGl2RenderingContext,
    program: WebGlProgram,
    input_bindings: js_bindings::InputBinding,
    u_location_names: JsValue,
    canvas_elem: HtmlCanvasElement,
) {
    util::set_panic_hook();

    let u_location_names: Vec<String> = serde_wasm_bindgen::from_value(u_location_names)
        .expect("Location names should be strings.");
    
    let mut game_loop: spin::GameLoop = spin::GameLoop::empty(context);
    game_loop.setup(std::rc::Rc::new(program), &u_location_names);
    game_loop.bind_canvas(Some(&canvas_elem));
    game_loop.step(input_bindings);
}
