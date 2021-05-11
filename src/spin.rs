use std::collections::HashMap;
use std::rc::Rc;

use web_sys::{WebGl2RenderingContext as Gl, WebGlProgram, WebGlUniformLocation};

use crate::js_bindings::InputBinding;
use crate::rendering::{RenderItem, RenderableQueues, Renderer, ProgramData};
use crate::transform::Transform;
use crate::geometry;

type CallbackBox = Rc<dyn Fn(&mut WorldState)>;

pub trait Callback: FnOnce(&mut WorldState) {}

pub struct WorldState {
    pub frame_idx: u64,
    pub inputs: Option<InputBinding>,
    pub renderables: RenderableQueues,
    pub transforms: Vec<Rc<Transform>>,
    new_step_logic_cbs: Vec<CallbackBox>,
}

impl WorldState {
    pub fn empty() -> Self {
        WorldState {
            frame_idx: 0,
            inputs: None,
            renderables: RenderableQueues::new(),
            transforms: Vec::new(),
            new_step_logic_cbs: Vec::new(),
        }
    }

    pub fn transfer_callbacks(&mut self, out_vec: &mut Vec<CallbackBox>) {
        out_vec.extend(self.new_step_logic_cbs.drain(0..));
    }

    pub fn add_step_cb(&mut self, cb: CallbackBox) {
        self.new_step_logic_cbs.push(cb);
    }
}

pub struct GameLoop<'a> {
    renderer: Renderer<'a>,
    pub state: WorldState,
    step_logic_cbs: Vec<Rc<dyn Fn(&mut WorldState)>>,
    canvas_elem: Option<&'a web_sys::HtmlCanvasElement>,
}

impl<'a> GameLoop<'a> {
    pub fn empty(ctx: &'a Gl) -> Self {
        Self {
            renderer: Renderer { ctx },
            state: WorldState::empty(),
            step_logic_cbs: Vec::new(),
            canvas_elem: None,
        }
    }

    /// Set the canvas object to render to.
    pub fn bind_canvas(&mut self, canvas_elem: Option<&'a web_sys::HtmlCanvasElement>) {
        self.canvas_elem = canvas_elem;
    }

    pub fn register_step_logic_fn(&mut self, f: Rc<dyn Fn(&mut WorldState)>) {
        self.state.add_step_cb(f);
        self.update_step_logic_cbs();
    }

    pub fn setup(&mut self, program: Rc<WebGlProgram>, location_names: &[String]) {
        let program_data = self.renderer.make_program_data(program, location_names);
        let cb: Rc<dyn Fn(&mut WorldState)> = Rc::new(move |state| {
            square_follower(state, program_data.clone());
        });
        self.register_step_logic_fn(cb);
    }

    fn update_step_logic_cbs(&mut self) {
        self.state.transfer_callbacks(&mut self.step_logic_cbs);
    }

    pub fn start(&mut self) {}

    pub fn step(&mut self, inputs: InputBinding) {
        self.update_step_logic_cbs();
        self.state.inputs = Some(inputs);
        for f in self.step_logic_cbs.iter_mut() {
            f(&mut self.state);
        }
        self.renderer.render_all(&self.state.renderables);
        self.state.frame_idx += 1;
    }
}

fn square_follower(state: &mut WorldState, program_data: ProgramData) {
    let transform = Rc::new(Transform::identity());
    state.transforms.push(transform.clone());
    state.renderables.push_forward_queue(Rc::new(
        RenderItem::new(program_data, transform, &geometry::new_square(1.)),
    ));
}
