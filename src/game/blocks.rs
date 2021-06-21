use na::Vector3;

use crate::maths_utils;
use crate::mesh;
use crate::rendering::ProgramData;
use crate::transform::Transform;
use crate::world_object::{
    MeshComponent, RenderComponent, WorldObject3DInit, WorldObjectId,
};
use crate::world_state::WorldState;

struct MovingBlockBehavior {
    self_id: WorldObjectId,
}

const BLOCK_SIZE: f32 = 1.0;

pub fn make_blocks(program_data: ProgramData, ws: &mut WorldState) -> Result<(), String> {
    for i in 0..10 {
        for j in 0..10 {
            let mut tf = Transform::identity();
            tf.set_position(Vector3::new(
                ((5 - i) as f32) * BLOCK_SIZE,
                0.0,
                ((5 - j) as f32) * BLOCK_SIZE,
            ));
            tf.set_scale(Vector3::new(0.8, 0.8, 0.8));
            ws.add_world_obj(
                WorldObject3DInit {
                    tf,
                    mesh: Some(MeshComponent {
                        data: mesh::wavefront_obj::into_vertex_vec().unwrap(),
                    }),
                    render: Some(RenderComponent {
                        gl_program_data: program_data.clone(),
                        renderer: ws.get_renderer().ok_or("Failed to make blocks")?,
                    }),
                    ..Default::default()
                }
                .init(),
            );
        }
    }
    Ok(())
}
