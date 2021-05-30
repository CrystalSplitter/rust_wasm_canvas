extern crate console_error_panic_hook;
extern crate nalgebra as na;
extern crate num;
extern crate serde_wasm_bindgen;
extern crate wasm_bindgen;
extern crate web_sys;

use once_cell::sync::Lazy;
use std::borrow::Borrow;
use std::sync::{Arc, Mutex};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::*;

mod geometry;
mod inputs;
mod js_bindings;
mod maths_utils;
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

static WORLD_KILLER: Lazy<Mutex<Box<dyn (Fn()) + Sync + Send>>> =
    Lazy::new(|| Mutex::new(Box::new(|| {})));
static EXIT_ERROR: Lazy<Mutex<Option<String>>> = Lazy::new(Mutex::default);

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
    let mut game_loop: spin::GameLoop = spin::GameLoop::empty();
    game_loop.bind_canvas(canvas_elem, context);
    let r = game_loop.setup(std::rc::Rc::new(program), &u_location_names);
    match r {
        Err(e) => e,
        _ => {
            match recursive_loop(game_loop) {
                Ok((closure, handle)) => {
                    let mut killer = WORLD_KILLER.lock().unwrap();
                    *killer = Box::new(move || {
                        let window = window().expect("No global `window` exists. Unable to kill closure.");
                        window.clear_interval_with_handle(handle);
                        let exit_space = EXIT_ERROR.lock().unwrap();
                        if let Some(x) = exit_space.borrow().as_ref() {
                            console::error_1(&x.into())
                        }
                    });
                    closure.forget();
                    "OK".into()
                },
                Err(e) => e,
            }
        }
    }
}

type IntervalType = (Closure<dyn FnMut()>, i32);

fn recursive_loop(game_loop: spin::GameLoop) -> Result<IntervalType, String> {
    let window = window().expect("No global `window` exists. Exiting.");
    let game_loop = Arc::new(Mutex::new(game_loop));
    let closure = Closure::wrap(Box::new(move || {
        let stepped = game_loop.clone().lock().unwrap().step();
        if let Err(e) = stepped {
            {
                // Set the exit error to print out.
                let mut exit_space = EXIT_ERROR.lock().unwrap();
                *exit_space = Some(e);
            }
            // Kill the spawned thread.
            let killer = WORLD_KILLER.lock().unwrap();
            killer.borrow()();
        }
    }) as Box<dyn FnMut()>);
    let r = window.set_interval_with_callback_and_timeout_and_arguments_0(
        closure.as_ref().unchecked_ref(),
        // 1000.0 for Milliseconds.
        (1000.0 / FRAME_RATE_CAP) as i32,
    );
    match r {
        Ok(handle) => {
            Ok((closure, handle))
        }
        Err(e) => {
            let e_string: String = e.as_string().unwrap_or_else(|| "[Unable to display error]".into());
            Err("Unable to make interval: ".to_string() + &e_string)
        }
    }
}
