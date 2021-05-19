extern crate console_error_panic_hook;
extern crate nalgebra as na;
extern crate serde_wasm_bindgen;
extern crate wasm_bindgen;
extern crate web_sys;

use std::sync::{Arc, Mutex};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::*;

mod geometry;
mod inputs;
mod js_bindings;
mod rendering;
mod spin;
mod transform;
mod util;

const FRAME_RATE_CAP: f32 = 60.0;

#[wasm_bindgen]
pub fn get_transform_mat(x: f32, y: f32) -> Vec<f32> {
    let v: na::Matrix3<f32> = na::Matrix3::identity().append_translation(&na::Vector2::new(x, y));
    v.as_slice().to_vec()
}

#[wasm_bindgen]
pub fn bind_game(
    context: web_sys::WebGl2RenderingContext,
    program: WebGlProgram,
    u_location_names: JsValue,
    canvas_elem: HtmlCanvasElement,
) -> String {
    // Useful for debugging.
    util::set_panic_hook();

    // Get list of locations as a 
    let u_location_names: Vec<String> = serde_wasm_bindgen::from_value(u_location_names)
        .expect("Location names should be strings.");
    let mut game_loop: spin::GameLoop = spin::GameLoop::empty(context);
    game_loop.bind_canvas(canvas_elem);
    let r = game_loop.setup(std::rc::Rc::new(program), &u_location_names);
    match r {
        Err(e) => e,
        _ => {
            recursive_loop(game_loop);
            "OK".into()
        }
    }
}

fn recursive_loop(game_loop: spin::GameLoop) {
    let window = window().expect("No global `window` exists. Exiting.");
    let game_loop = Arc::new(Mutex::new(game_loop));
    let closure = Closure::wrap(Box::new(move || {
        game_loop.clone().lock().unwrap().step();
    }) as Box<dyn FnMut()>);
    let success = window.set_interval_with_callback_and_timeout_and_arguments_0(
        closure.as_ref().unchecked_ref(),
        // 1000.0 for Milliseconds.
        (1000.0 / FRAME_RATE_CAP) as i32,
    );
    closure.forget();
    if success.is_err() {
        console::error_1(&"Unable to add the set interval callback".into());
    }
}
