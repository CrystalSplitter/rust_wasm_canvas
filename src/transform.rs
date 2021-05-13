use na::{Matrix3, Matrix4, Rotation3, Vector2, Vector3};

#[derive(Debug, Clone)]
pub struct Transform {
    translation: Vector3<f32>,
    rotation: Vector3<f32>,
    scale: Vector3<f32>,
}

impl Transform {
    pub fn identity() -> Self {
        Transform {
            translation: Vector3::new(0., 0., 0.),
            rotation: Vector3::new(0., 0., 0.),
            scale: Vector3::new(1., 1., 1.),
        }
    }

    pub fn new(translation: Vector3<f32>, rotation: Vector3<f32>, scale: Vector3<f32>) -> Self {
        Transform {
            translation,
            rotation,
            scale,
        }
    }

    pub fn to_mat3(&self) -> na::Matrix3<f32> {
        Matrix3::new_nonuniform_scaling(&Vector2::new(self.scale[0], self.scale[1]))
            .append_translation(&Vector2::new(self.translation[0], self.translation[1]))
    }

    pub fn to_mat3_vec(&self) -> Vec<f32> {
        self.to_mat3().iter().copied().collect()
    }

    pub fn to_mat4(&self) -> na::Matrix4<f32> {
        Matrix4::new_nonuniform_scaling(&self.scale).append_translation(&self.translation)
    }
}
