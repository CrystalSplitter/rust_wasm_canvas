use nalgebra as na;

pub type VertArray = js_sys::Float32Array;

pub fn new_square(scale: f32) -> Vec<f32> {
    vec![0, 0, 1, 0, 0, 1, 1, 0, 0, 1, 1, 1]
        .iter()
        .map(|&x| (x as f32) * scale)
        .collect()
}

pub fn new_cube(scale: f32) -> Vec<f32> {
    vec![
        // Bottom face
        0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 1, 0, 1, 0, 0, 1, 1, 0, // Top face
        0, 0, 1, 1, 0, 1, 0, 1, 1, 0, 1, 1, 1, 0, 1, 1, 1, 1,
    ]
    .iter()
    .map(|&x| (x as f32) * scale)
    .collect()
}
