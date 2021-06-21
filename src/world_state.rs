use std::cell::RefCell;
use std::rc;
use std::rc::Rc;
use std::sync::Arc;

use slab::Slab;

use crate::inputs;
use crate::rendering::*;
use crate::steppables::Steppable;
use crate::world_object::{WorldObject3D, WorldObject3DInit, WorldObjectId};
use crate::util::{Rfc, Wfc};

pub enum RendQueueType {
    Fwd,
    Rev,
}

/// Type alias so we can change the provided type all together.
type InputT = inputs::InputBinding;
type CanvasT = web_sys::HtmlCanvasElement;

type WorldSteppable = dyn Steppable<WorldState>;

pub struct WorldState {
    pub delta_time: f32,

    canvas: Option<Arc<CanvasT>>,
    frame_count: u64,
    world_objs: Slab<WorldObject3D>,
    scripted_components: Vec<Rfc<WorldSteppable>>,
    inputs: Option<InputT>,
    renderables: RenderableQueues,
    renderer: Option<Arc<dyn Renderer>>,
}

impl WorldState {
    pub fn new() -> Self {
        WorldState {
            frame_count: 0,
            inputs: None,
            renderables: RenderableQueues::new(),
            world_objs: Default::default(),
            scripted_components: vec![],
            renderer: None,
            canvas: None,
            delta_time: 1.0 / 60.0,
        }
    }

    pub fn set_renderer(&mut self, renderer: impl Renderer + 'static) -> &mut Self {
        self.renderer = Some(Arc::new(renderer));
        self
    }

    pub fn get_renderer(&self) -> Option<Arc<dyn Renderer>> {
        self.renderer.clone()
    }

    pub fn set_canvas(&mut self, canvas: Option<Arc<CanvasT>>) -> &mut Self {
        self.canvas = canvas;
        self
    }

    pub fn get_canvas(&self) -> Option<Arc<CanvasT>> {
        self.canvas.clone()
    }

    pub fn get_frame_count(&self) -> u64 {
        self.frame_count
    }

    pub fn get_inputs(&self) -> &Option<InputT> {
        &self.inputs
    }

    pub fn set_inputs(&mut self, new_inputs: Option<InputT>) {
        self.inputs = new_inputs;
    }

    pub fn inc_frame_count(&mut self) -> u64 {
        self.frame_count += 1;
        self.frame_count
    }

    pub fn get_rendqueue(&self) -> &RenderableQueues {
        &self.renderables
    }

    pub fn get_rendqueue_mut(&mut self) -> &mut RenderableQueues {
        &mut self.renderables
    }

    fn push_rendqueue(&mut self, queue_type: RendQueueType, item: Rc<RenderItem>) {
        match queue_type {
            RendQueueType::Fwd => {
                self.renderables.push_forward_queue(item);
            }
            RendQueueType::Rev => {
                self.renderables.push_reverse_queue(item);
            }
        }
    }

    pub fn get_world_obj_mut(&mut self, id: &WorldObjectId) -> Option<&mut WorldObject3D> {
        self.world_objs.get_mut(id.0)
    }

    pub fn get_world_obj(&self, id: &WorldObjectId) -> Option<&WorldObject3D> {
        self.world_objs.get(id.0)
    }

    pub fn new_world_obj(&mut self) -> WorldObjectId {
        let obj = WorldObject3DInit {
            ..Default::default()
        }
        .init();
        WorldObjectId(self.world_objs.insert(obj))
    }

    pub fn add_world_obj(&mut self, obj: WorldObject3D) -> WorldObjectId {
        if let Some(item) = obj.render_item.clone() {
            self.push_rendqueue(RendQueueType::Fwd, item)
        }
        let children = obj.get_children_ids().to_owned();
        let new_id = WorldObjectId(self.world_objs.insert(obj));
        for c in children {
            if let Some(child) = self.get_world_obj_mut(&c) {
                child.set_parent_id(Some(new_id.clone()));
            }
        }
        self.get_world_obj_mut(&new_id).unwrap().self_id = Some(new_id);
        new_id
    }

    pub fn scripted_components(&self) -> Vec<Rc<RefCell<WorldSteppable>>> {
        self.scripted_components.clone()
    }

    pub fn add_scripted_component(&mut self, s: impl Steppable<Self> + 'static) {
        self.scripted_components.push(Rc::new(RefCell::new(s)));
    }

    pub fn run_steps(
        self: &mut WorldState,
        steppables: &mut Vec<Rc<RefCell<WorldSteppable>>>,
    ) -> Result<(), String> {
        for s in steppables.iter_mut() {
            s.borrow_mut().step(self).or_else(|e| e.translate())?;
        }
        Ok(())
    }

    pub fn run_fixed_steps(
        self: &mut WorldState,
        steppables: &mut Vec<Rc<RefCell<WorldSteppable>>>,
    ) -> Result<(), String> {
        for s in steppables.iter_mut() {
            s.borrow_mut().fixed_step(self).or_else(|e| e.translate())?;
        }
        Ok(())
    }

    pub fn run_late_steps(
        self: &mut WorldState,
        steppables: &mut Vec<Rc<RefCell<WorldSteppable>>>,
    ) -> Result<(), String> {
        for s in steppables.iter_mut() {
            s.borrow_mut().late_step(self).or_else(|e| e.translate())?;
        }
        Ok(())
    }
}
