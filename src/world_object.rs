use std::cell::RefCell;
use std::rc::{Rc, Weak};
use std::sync::Arc;

use web_sys::WebGl2RenderingContext as Gl;

use crate::rendering::Renderer;
use crate::rendering::*;
use crate::steppables::Steppable;
use crate::transform::Transform;
use crate::world_state::WorldState;

#[derive(Debug, Copy, Clone, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub struct WorldObjectId(pub usize);

pub struct MeshComponent {
    pub data: Vec<f32>,
}

pub struct Material {
    pub color: (f32, f32, f32, f32),
}

pub struct RenderComponent {
    pub gl_program_data: ProgramData,
    pub renderer: Arc<dyn Renderer>,
    pub material: Material,
}

pub struct WorldObject3DInit {
    pub tf: Transform,
    pub mesh: Option<MeshComponent>,
    pub render: Option<RenderComponent>,
    pub render_item: Option<RenderItem>,
    pub scripts: Vec<Rc<dyn Steppable<WorldState>>>,
    pub children_ids: Vec<WorldObjectId>,
    pub parent_id: Option<WorldObjectId>,
    #[doc(hidden)]
    pub __non_exh: (),
}

impl WorldObject3DInit {
    pub fn init(self) -> WorldObject3D {
        let tf_rc = Rc::new(RefCell::new(self.tf));
        let obj = WorldObject3D {
            render_item: match (self.render, self.mesh, self.render_item) {
                (Some(rend), Some(m), None) => {
                    let ctx = rend.renderer.get_ctx();
                    let mut color_data = vec![];
                    for _ in 0..m.data.len() / 3 {
                        color_data.push(rend.material.color.0);
                        color_data.push(rend.material.color.1);
                        color_data.push(rend.material.color.2);
                        color_data.push(rend.material.color.3);
                    }
                    let buffer_info = BufferInfo::new(ctx.clone())
                        .add_buffer(
                            "a_position".into(),
                            m.data,
                            BufferSettings::new(3, Gl::FLOAT),
                        )
                        .add_buffer(
                            "a_color".into(),
                            color_data,
                            BufferSettings::new(4, Gl::FLOAT),
                        );
                    let vao = ctx.create_vertex_array();
                    Some(Rc::new(
                        RenderItem::builder()
                            .buffer_info(buffer_info)
                            .program_data(rend.gl_program_data)
                            .tf(tf_rc.clone())
                            .vao(vao.unwrap())
                            .build()
                            .unwrap(),
                    ))
                }
                (None, None, Some(render_item)) => Some(Rc::new(render_item)),
                _ => None,
            },
            self_id: None,
            tf_rc,
            children: self.children_ids,
            parent: None,
            name: "".to_string(),
        };
        obj
    }
}

impl Default for WorldObject3DInit {
    fn default() -> Self {
        Self {
            tf: Transform::identity(),
            render: None,
            mesh: None,
            scripts: vec![],
            children_ids: vec![],
            parent_id: None,
            render_item: None,
            __non_exh: (),
        }
    }
}

pub struct WorldObject3D {
    pub tf_rc: Rc<RefCell<Transform>>,
    pub render_item: Option<Rc<RenderItem>>,
    pub name: String,
    pub(super) self_id: Option<WorldObjectId>,
    children: Vec<WorldObjectId>,
    parent: Option<WorldObjectId>,
}

impl WorldObject3D {
    pub fn get_parent<'a>(&self, state: &'a WorldState) -> Option<&'a WorldObject3D> {
        self.parent
            .and_then(|p_id| state.get_world_obj(&p_id))
            .and_then(|p| Some(p))
    }

    pub fn get_parent_mut<'a>(&self, state: &'a mut WorldState) -> Option<&'a mut WorldObject3D> {
        if let Some(parent) = self.parent {
            state.get_world_obj_mut(&parent)
        } else {
            None
        }
    }

    pub fn set_parent_id(&mut self, parent: Option<WorldObjectId>) -> &mut Self {
        self.parent = parent;
        self
    }

    pub fn get_child_id_by_name(&self, state: &WorldState, name: &str) -> Option<WorldObjectId> {
        for id in self.children.iter() {
            let is_obj = state
                .get_world_obj(id)
                .and_then(|c| Some(c.name == name))
                .unwrap_or(false);
            if is_obj {
                return Some(id.to_owned());
            }
        }
        None
    }

    pub fn get_children_ids(&self) -> &Vec<WorldObjectId> {
        &self.children
    }
}

#[cfg(Test)]
mod Test {
    fn test_init() {
        let state: WorldState = WorldState::new();
        let obj: WorldObject3D = WorldObject3DInit {
            ..Default::default()
        }
        .init();
        state.add_world_object(obj);
    }
}
