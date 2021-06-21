use crate::js_bindings;
use wavefront_obj::obj::{Object, Primitive};

// Parsing produces an ObjSet
// ObjSets have Objs
// Objs have a geometry field which has shapes & vertices
// shapes have a Geometry
// Geometries have a primitive
// Primitives are either Points, Lines, or Triangles
// Points, Lines, and Triangles refer to vertices of the Obj

pub fn into_vertex_vec() -> Result<Vec<f32>, String> {
    let mut vert_tups: Vec<(f64, f64, f64)> = Vec::new();
    wavefront_obj::obj::parse(mesh_string())
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
    let bytes = include_bytes!("/home/crystal/Desktop/cube.obj");
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
