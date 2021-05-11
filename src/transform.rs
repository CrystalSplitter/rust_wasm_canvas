use na::{Matrix3, Rotation3, Vector2};

#[derive(Debug, Clone)]
pub struct Transform {
    translation: Vec<f32>,
    rotation: Vec<f32>,
    scale: Vec<f32>,
}

impl Transform {
    pub fn identity() -> Self {
        Transform {
            translation: vec![0., 0., 0.],
            rotation: vec![0., 0., 0.],
            scale: vec![1., 1., 1.],
        }
    }

    pub fn to_f32_vec(&self) -> Vec<f32> {
        vec![]
    }
}
