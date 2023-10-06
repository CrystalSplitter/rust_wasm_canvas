use crate::js_bindings;

use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use wavefront_obj::obj::{Object, Primitive};
use web_sys::{Request, RequestInit, RequestMode, Response};

// Parsing produces an ObjSet
// ObjSets have Objs
// Objs have a geometry field which has shapes & vertices
// shapes have a Geometry
// Geometries have a primitive
// Primitives are either Points, Lines, or Triangles
// Points, Lines, and Triangles refer to vertices of the Obj

pub fn into_vertex_vec(file_data: &str) -> Result<Vec<f32>, String> {
    let mut vert_tups: Vec<(f64, f64, f64)> = Vec::new();
    wavefront_obj::obj::parse(file_data)
        .map(|obj_set| {
            let obj = &obj_set.objects[0];
            let geo = &obj.geometry[0];
            for shape in &geo.shapes {
                match shape.primitive {
                    Primitive::Triangle(p1, p2, p3) => {
                        vert_tups.push(get_vertex_pos(obj, p1.0));
                        vert_tups.push(get_vertex_pos(obj, p2.0));
                        vert_tups.push(get_vertex_pos(obj, p3.0));
                    }
                    Primitive::Line(p1, p2) => {
                        js_bindings::error(&format!("Line found! {:?} {:?}", p1, p2));
                    }
                    Primitive::Point(p) => {
                        js_bindings::error(&format!("Point found! {:?}", p));
                    }
                }
            }
            concat_vert_tups(&vert_tups)
        })
        .map_err(|e| e.to_string())
}

pub fn mesh_string() -> String {
    let bytes = include_bytes!("/home/crystal/Desktop/suzanne.obj");
    String::from_utf8_lossy(bytes).into_owned()
}

fn get_vertex_pos(obj: &Object, v_id: usize) -> (f64, f64, f64) {
    let vert = obj.vertices[v_id];
    (vert.x, vert.y, vert.z)
}

fn concat_vert_tups(vert_tups: &[(f64, f64, f64)]) -> Vec<f32> {
    let mut out_vec: Vec<f32> = Vec::new();
    vert_tups
        .iter()
        .fold(&mut out_vec, |acc: &mut Vec<_>, &(x, y, z)| {
            acc.push(x as f32);
            acc.push(y as f32);
            acc.push(z as f32);
            acc
        });
    out_vec
}

pub fn get_mesh_data_from_url(url: String) -> std::sync::Arc<std::sync::Mutex<Option<String>>> {
    let out = std::sync::Arc::new(std::sync::Mutex::new(None));
    let out_clone = out.clone();
    wasm_bindgen_futures::spawn_local(async move {
        let result = async_get_mesh_data_from_url(&url).await;
        match (out_clone.try_lock(), result) {
            (Ok(mut lock), Ok(r)) => {
                *lock = Some(r);
            }
            (Ok(mut lock), Err(e)) => {
                js_bindings::error(&e);
                *lock = None;
            }
            (Err(e), _) => js_bindings::error(&format!(
                "Unable to acquire lock for mesh data output arc: {}",
                e.to_string()
            )),
        }
    });
    out
}

pub async fn async_get_mesh_data_from_url(url: &str) -> Result<String, String> {
    let mut opts = RequestInit::new();
    opts.method("GET");
    opts.mode(RequestMode::Cors);

    let request: Request = Request::new_with_str_and_init(&url, &opts).map_err(|x: JsValue| {
        x.as_string()
            .unwrap_or("Couldn't convert request error to string".into())
    })?;
    let window = web_sys::window().unwrap();
    let resp_value: JsValue = JsFuture::from(window.fetch_with_request(&request))
        .await
        .map_err(|x| x.as_string().unwrap_or("Couldn't convert to string".into()))?;
    let resp_text: JsValue = JsFuture::from(
        resp_value
            .dyn_into::<Response>()
            .expect("Not an instance of Response")
            .text()
            .unwrap(),
    )
    .await
    .map_err(|e| e.as_string().unwrap_or("Couldn't convert error".into()))?;
    Ok(resp_text
        .as_string()
        .expect("Could not convert response text to String"))
}
