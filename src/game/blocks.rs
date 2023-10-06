use std::sync::{Arc, Mutex};

use na::Vector3;

use crate::maths_utils;
use crate::mesh;
use crate::rendering;
use crate::rendering::ProgramData;
use crate::rendering::RenderLineBuilder;
use crate::steppables::StepError;
use crate::steppables::Steppable;
use crate::transform::Transform;
use crate::world_object::{
    Material, MeshComponent, RenderComponent, WorldObject3DInit, WorldObjectId,
};
use crate::world_state::WorldState;

struct MovingBlockBehavior {
    self_id: WorldObjectId,
}

const GRID_SIZE: f32 = 1.0;
const BLOCK_SIZE: f32 = 0.3;

#[derive(Debug, Clone)]
pub struct BlockBehavior {
    pub program_data: ProgramData,
    pub mesh_data: String,
}

impl BlockBehavior {
    pub fn make_blocks(
        &self,
        program_data: &ProgramData,
        mesh_data: &str,
        ws: &mut WorldState,
    ) -> Result<(), String> {
        let width = 30;
        let length = 30;

        let renderer = ws.get_renderer().expect("Could not get renderer");
        let mut line_builder = RenderLineBuilder::new()
            .program_data(program_data.clone())
            .renderer(renderer.as_ref())
            //.default_color(color)
            .vao(
                renderer
                    .get_ctx()
                    .create_vertex_array()
                    .ok_or("Could not create VAO")?,
            );

        for i in 0..width {
            for j in 0..length {
                let mut tf = Transform::identity();
                let x_pos = ((width / 2 - i) as f32) * GRID_SIZE;
                let y_pos = ((length / 2 - j) as f32) * GRID_SIZE;
                let z_pos = 0.;
                tf.set_position(Vector3::new(x_pos, y_pos, z_pos));
                tf.set_scale(Vector3::new(BLOCK_SIZE, BLOCK_SIZE, BLOCK_SIZE));
                ws.add_world_obj(
                    WorldObject3DInit {
                        tf,
                        mesh: Some(MeshComponent {
                            data: mesh::wavefront_obj::into_vertex_vec(mesh_data).unwrap(),
                        }),
                        render: Some(RenderComponent {
                            gl_program_data: program_data.clone(),
                            renderer: ws.get_renderer().ok_or("Failed to make blocks")?,
                            material: Material {
                                color: (1.0, 0.6, 1.0, 1.0),
                            },
                        }),
                        ..Default::default()
                    }
                    .init(),
                );

                line_builder = add_line(
                    Vector3::new(x_pos, y_pos, z_pos),
                    Vector3::new(x_pos + 1.0, y_pos, z_pos),
                    (1., 0., 0., 1.),
                    line_builder,
                )?;
                line_builder = add_line(
                    Vector3::new(x_pos, y_pos, z_pos),
                    Vector3::new(x_pos, y_pos + 1.0, z_pos),
                    (0., 1., 0., 1.),
                    line_builder,
                )?;
                line_builder = add_line(
                    Vector3::new(x_pos, y_pos, z_pos),
                    Vector3::new(x_pos, y_pos, z_pos + 1.0),
                    (0., 0., 1., 1.),
                    line_builder,
                )?;
            }
        }
        ws.add_world_obj(
            WorldObject3DInit {
                tf: Transform::identity(),
                render_item: Some(line_builder.build().expect("Could not build line")),
                ..Default::default()
            }
            .init(),
        );
        Ok(())
    }
}

impl Steppable<WorldState> for BlockBehavior {
    fn start(&mut self, ws: &mut WorldState) -> Result<(), StepError<String>> {
        self.make_blocks(&self.program_data, &self.mesh_data, ws)
            .map_err(|e| StepError::Fatal(e))
    }
}

fn add_line<R: rendering::Renderer + ?Sized>(
    start: Vector3<f32>,
    end: Vector3<f32>,
    color: (f32, f32, f32, f32),
    line_builder: RenderLineBuilder<R>,
) -> Result<RenderLineBuilder<R>, String> {
    Ok(line_builder.add_line_with_color((start, end), color))
}
