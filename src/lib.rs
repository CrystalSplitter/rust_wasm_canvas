extern crate console_error_panic_hook;
extern crate dyn_clone;
extern crate nalgebra as na;
extern crate num;
extern crate rand;
extern crate serde;
extern crate serde_wasm_bindgen;
extern crate slab;
extern crate wasm_bindgen;
extern crate wasm_bindgen_futures;
extern crate web_sys;

use once_cell::sync::Lazy;
use std::borrow::Borrow;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use wasm_bindgen::prelude::*;
use web_sys::*;

use js_bindings::{ergonomic_interval, IntervalType};

mod geometry;
mod inputs;
mod js_bindings;
mod maths_utils;
mod mesh;
mod rendering;
mod rigidbody;
mod spin;
mod steppables;
mod transform;
#[macro_use]
mod util;
mod game;
mod shader_config;
mod world_object;
mod world_state;

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
pub struct PreloadedGameData {

}

#[wasm_bindgen]
pub async fn bind_game(
    context: web_sys::WebGl2RenderingContext,
    program: WebGlProgram,
    u_location_names: JsValue,
    canvas_elem: HtmlCanvasElement,
    preloaded_game_data: PreloadedGameData,
) -> String {
    // Useful for debugging.
    set_panic_hook();
    match bootstrap(context, program, canvas_elem) {
        Err(e) => {
            js_bindings::error(&e);
            e
        }
        _ => "Ok".into(),
    }
}

fn bootstrap(
    context: web_sys::WebGl2RenderingContext,
    program: WebGlProgram,
    canvas_elem: HtmlCanvasElement,
) -> Result<(), String> {
    // Get list of locations as a vector.
    /*
    let u_location_names: Vec<String> = serde_wasm_bindgen::from_value(u_location_names)
        .expect("Location names should be strings.");
    */
    let mut game_loop: spin::GameLoop = spin::GameLoop::empty();
    let ctx_rc = Rc::new(context);
    game_loop.bind_canvas(canvas_elem, ctx_rc);
    game_loop.add_load_in_mesh("assets/cube.obj".into());
    game_loop.add_load_in_mesh("assets/cottage_1.obj".into());
    game_loop.add_gl_program(
        "vertex_color".into(),
        program,
        &[
            "u_transformationMatrix".into(),
            "a_position".into(),
            "a_color".into(),
        ],
    );
    ergonomic_interval((1.0)game_loop.load_in();
    game_loop.setup()?;
    game_loop.start()?;
    let (closure, handle) = recursive_loop(game_loop)?;
    let mut killer = WORLD_KILLER.lock().unwrap();
    *killer = Box::new(move || {
        let window = window().expect("No global `window` exists. Unable to kill closure.");
        window.clear_interval_with_handle(handle);
        let exit_space = EXIT_ERROR.lock().unwrap();
        if let Some(x) = exit_space.borrow().as_ref() {
            js_bindings::error(x)
        }
    });
    closure.forget();
    Ok(())
}

fn recursive_loop(game_loop: spin::GameLoop) -> Result<IntervalType, String> {
    let game_loop = Arc::new(Mutex::new(game_loop));
    ergonomic_interval((1000.0 / FRAME_RATE_CAP) as i32, move || {
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
    })
}

pub fn set_panic_hook() {
    // When the `console_error_panic_hook` feature is enabled, we can call the
    // `set_panic_hook` function at least once during initialization, and then
    // we will get better error messages if our code ever panics.
    //
    // For more details see
    // https://github.com/rustwasm/console_error_panic_hook#readme
    //#[cfg(feature = "console_error_panic_hook")]
    let killer = WORLD_KILLER.lock().unwrap();
    killer.borrow()();
    console_error_panic_hook::set_once();
}
