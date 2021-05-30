use na::{Matrix3, Matrix4, Rotation3, Vector2, Vector3, Vector4};
use crate::maths_utils::{EulerAngles3D};

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

    pub fn set_position(&mut self, position: Vector3<f32>) -> &mut Self {
        self.translation = position;
        self
    }
    
    pub fn get_position(&self) -> Vector3<f32> {
        self.translation
    }

    /// Set rotation in Euler coordinates (pitch, roll, yaw) in radians.
    pub fn set_euler_rotation_raw(&mut self, rot: Vector3<f32>) -> &mut Self {
        self.rotation = rot;
        self
    }
    
    /// Set rotation in Euler coordinates (pitch, roll, yaw) in radians.
    pub fn set_euler_rotation(&mut self, rot: EulerAngles3D<f32>) -> &mut Self {
        self.set_euler_rotation_raw(Vector3::new(rot.roll.as_rad(), rot.pitch.as_rad(), rot.yaw.as_rad()))
    }
    
    pub fn set_rotation(&mut self, rot: Vector4<f32>) -> &mut Self {
        //self.rotation = rot;
        panic!();
    }

    pub fn set_scale(&mut self, scale: Vector3<f32>) -> &mut Self {
        self.scale = scale;
        self
    }
    
    pub fn get_scale(&self) -> Vector3<f32> {
        self.scale
    }

    pub fn to_mat3(&self) -> na::Matrix3<f32> {
        Matrix3::new_nonuniform_scaling(&Vector2::new(self.scale[0], self.scale[1]))
            .append_translation(&Vector2::new(self.translation[0], self.translation[1]))
    }

    pub fn to_mat3_vec(&self) -> Vec<f32> {
        self.to_mat3().iter().copied().collect()
    }

    pub fn to_mat4(&self) -> na::Matrix4<f32> {
        (Matrix4::new_nonuniform_scaling(&self.scale)
            * Rotation3::from_euler_angles(self.rotation[0], self.rotation[1], self.rotation[2])
                .to_homogeneous())
        .append_translation(&self.translation)
    }

    pub fn to_mat4_vec(&self) -> Vec<f32> {
        self.to_mat4().iter().copied().collect()
    }
}
