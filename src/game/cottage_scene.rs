use std::sync::{Arc, Mutex};

use crate::mesh::wavefront_obj::{get_mesh_data_from_url, into_vertex_vec};
use crate::spin::GameLoop;
use crate::steppables::{StepError, Steppable};
use crate::world_object::{
    Material, MeshComponent, RenderComponent, WorldObject3D, WorldObject3DInit,
};
use crate::world_state::WorldState;

impl Steppable<WorldState> for CottageMaker {
    fn start(&mut self, _state: &mut WorldState) -> Result<(), StepError<String>> {
        self.mesh_string_data = get_mesh_data_from_url("assets/cottage_1.obj".into());
        Ok(())
    }

    fn step(&mut self, state: &mut WorldState, _: &GameLoop) -> Result<(), StepError<String>> {
        if !self.made {
            if let Ok(mesh_lock) = self.mesh_string_data.try_lock() {
                if let Some(mesh_data) = mesh_lock.as_ref() {
                    let renderer = state
                        .get_renderer()
                        .ok_or(StepError::Recover("Failed to get renderer".into()))?;
                    state.add_world_obj(
                        WorldObject3DInit {
                            mesh: Some(MeshComponent {
                                data: into_vertex_vec(mesh_data)
                                    .expect("Could not parse wavefront obj data"),
                            }),
                            render: Some(RenderComponent {
                                renderer,
                                material: Material {
                                    color: (0.5, 0.5, 0.5, 1.0),
                                },
                                gl_program_data: panic!(),
                            }),
                            ..Default::default()
                        }
                        .init(),
                    );
                    self.made = true;
                }
            }
        }
        Ok(())
    }
}

struct CottageMaker {
    made: bool,
    mesh_string_data: Arc<Mutex<Option<String>>>,
}

impl Clone for CottageMaker {
    fn clone(&self) -> Self {
        Self {
            made: false,
            mesh_string_data: self.mesh_string_data.clone(),
        }
    }
}

/*
pub fn make_cottage(ws: &mut WorldState) -> Result<(), String> {
    let renderer = ws.get_renderer().expect("Could not get renderer");
    ws.add_world_obj(WorldObject3DInit {
        mesh: get_mesh_data_from_url("assets/cottage_1.obj"),
        ..Default::default()
    }.init());
    Ok(())
}
*/
