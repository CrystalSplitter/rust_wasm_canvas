use num::traits::{Float, FromPrimitive, NumOps};
use std::f32::consts::PI;

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Angle<T> {
    val: T,
}

impl<T: Float + NumOps + FromPrimitive> Angle<T> {
    pub fn from_deg(val: T) -> Angle<T> {
        Self::from_rad(val.to_radians())
    }

    pub fn from_rad(val: T) -> Angle<T> {
        let pi = FromPrimitive::from_f32(PI).unwrap();
        let double_pi = FromPrimitive::from_f32(2. * PI).unwrap();
        let mut wrapped = (val + pi) % double_pi;
        if wrapped < FromPrimitive::from_f32(0.).unwrap() {
            wrapped = wrapped + double_pi;
        }
        wrapped = wrapped - pi;
        Angle { val: wrapped }
    }

    pub fn as_rad(&self) -> T {
        self.val
    }
    pub fn as_deg(&self) -> T {
        self.val.to_degrees()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EulerAngles3D<T> {
    pub roll: Angle<T>,
    pub pitch: Angle<T>,
    pub yaw: Angle<T>,
}

impl<T: FromPrimitive + Float + NumOps> EulerAngles3D<T> {
    pub fn zeros() -> Self {
        Self {
            roll: Angle::from_rad(FromPrimitive::from_f32(0.).unwrap()),
            pitch: Angle::from_rad(FromPrimitive::from_f32(0.).unwrap()),
            yaw: Angle::from_rad(FromPrimitive::from_f32(0.).unwrap()),
        }
    }

    pub fn from_rad(roll: T, pitch: T, yaw: T) -> Self {
        Self {
            roll: Angle::from_rad(roll),
            pitch: Angle::from_rad(pitch),
            yaw: Angle::from_rad(yaw),
        }
    }

    pub fn from_deg(roll: T, pitch: T, yaw: T) -> Self {
        Self {
            roll: Angle::from_deg(roll),
            pitch: Angle::from_deg(pitch),
            yaw: Angle::from_deg(yaw),
        }
    }
}

/// Linearly interpolate between two points with a parametric value t.
pub fn lerp<T: Float + NumOps>(from: T, to: T, t: T) -> T {
    from + (t * (to - from))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn equality() {
        assert_eq!(Angle::from_deg(180.).as_rad(), Angle::from_rad(PI).as_rad());
        assert_eq!(
            Angle::from_deg(360.).as_rad(),
            Angle::from_rad(2. * PI).as_rad()
        );
        assert!((Angle::from_deg(360.).as_rad() - Angle::from_deg(-360.).as_rad()).abs() < 0.001);
        assert!(
            (Angle::from_deg(-360.).as_rad() - Angle::from_rad(2. * PI).as_rad()).abs() < 0.001
        );
    }
}
