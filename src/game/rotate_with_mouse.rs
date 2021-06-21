use std::cell::RefCell;
use std::rc::Rc;

use crate::inputs::Input;
use crate::maths_utils::EulerAngles3D;
use crate::steppables::{StepError, Steppable};
use crate::transform::Transform;
use crate::world_state::WorldState;

#[derive(Debug, Clone)]
pub struct RotateWithMouse {
    pub tf: Rc<RefCell<Transform>>,
}

impl Steppable<WorldState> for RotateWithMouse {
    fn step(&mut self, state: &mut WorldState) -> Result<(), StepError<String>> {
        match state.get_inputs() {
            Some(inputs) => {
                let mut tf = self
                    .tf
                    .try_borrow_mut()
                    .map_err(|_| StepError::Fatal("Could not borrow".into()))?;
                let x = -inputs.get_mouse_view_x() * 360.;
                let y = inputs.get_mouse_view_y() * 180. - 90.0;
                tf.set_euler_rotation(EulerAngles3D::from_deg(y, x, 0.));
                Ok(())
            }
            _ => Err(StepError::Recover("Could not get inputs".into())),
        }
    }
}
