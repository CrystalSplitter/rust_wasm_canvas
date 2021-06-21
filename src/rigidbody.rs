use crate::maths_utils::EulerAngles3D;
use na::Vector3;

pub struct Rigidbody3D {
    pub mass: f32,
    pub acceleration: Vector3<f32>,
    pub angular_acceleration: EulerAngles3D<f32>,
    pub des_velocity: Vector3<f32>,
    pub angular_velocity: Vector3<f32>,
    pub drag: f32,
    inst_force: Vector3<f32>,
    last_position: Vector3<f32>,
}

impl Rigidbody3D {
    pub fn new() -> Self {
        Self {
            mass: 1.,
            acceleration: Vector3::zeros(),
            angular_acceleration: EulerAngles3D::zeros(),
            des_velocity: Vector3::zeros(),
            angular_velocity: Vector3::zeros(),
            drag: 0.,
            inst_force: Vector3::zeros(),
            last_position: Vector3::zeros(),
        }
    }

    pub fn apply_force(&mut self, f: Vector3<f32>) {
        self.inst_force += f;
    }

    pub fn apply_angular_force(&mut self, f: EulerAngles3D<f32>) {}
}
