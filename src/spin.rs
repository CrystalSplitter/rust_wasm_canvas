use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use na::Vector3;
use wasm_bindgen::JsCast;
use web_sys::{console, HtmlCanvasElement, WebGl2RenderingContext as Gl, WebGlProgram};

use crate::geometry;
use crate::inputs;
use crate::inputs::{Input, InputBinding};
use crate::rendering::{RenderItem, RenderableQueues, Renderer};
use crate::transform::Transform;
//use crate::js_bindings::InputBinding;

pub enum RendQueueType {
    Fwd,
    Rev,
}

type CallbackBox<State> = Rc<dyn Fn(&mut State)>;
type InputT = InputBinding;

pub struct WorldState {
    frame_count: u64,
    inputs: Option<InputT>,
    renderables: RenderableQueues,
    transforms: Vec<Rc<RefCell<Transform>>>,
    new_step_logic_cbs: Vec<CallbackBox<Self>>,
}

impl WorldState {
    pub fn empty() -> Self {
        WorldState {
            frame_count: 0,
            inputs: None,
            renderables: RenderableQueues::new(),
            transforms: Vec::new(),
            new_step_logic_cbs: Vec::new(),
        }
    }

    pub fn transfer_callbacks(&mut self, out_vec: &mut Vec<CallbackBox<Self>>) {
        out_vec.extend(self.new_step_logic_cbs.drain(0..));
    }

    pub fn get_frame_count(&self) -> u64 {
        self.frame_count
    }

    pub fn add_step_cb(&mut self, cb: CallbackBox<Self>) {
        self.new_step_logic_cbs.push(cb);
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

    pub fn push_rendqueue(&mut self, queue_type: RendQueueType, item: Rc<RenderItem>) {
        match queue_type {
            RendQueueType::Fwd => {
                self.renderables.push_forward_queue(item);
            }
            RendQueueType::Rev => {
                self.renderables.push_reverse_queue(item);
            }
        }
    }

    pub fn get_transforms(&self) -> &Vec<Rc<RefCell<Transform>>> {
        &self.transforms
    }

    pub fn add_transform(&mut self, tf: Rc<RefCell<Transform>>) {
        self.transforms.push(tf);
    }
}

pub struct GameLoop {
    renderer: Renderer,
    state: Arc<Mutex<WorldState>>,
    step_logic_cbs: Vec<CallbackBox<WorldState>>,
    canvas_elem: Option<Arc<HtmlCanvasElement>>,
}

impl GameLoop {
    pub fn empty(ctx: Gl) -> Self {
        Self {
            renderer: Renderer { ctx },
            state: Arc::new(Mutex::new(WorldState::empty())),
            step_logic_cbs: Vec::new(),
            canvas_elem: None,
        }
    }

    /// Set the canvas object to render to.
    pub fn bind_canvas(&mut self, canvas_elem: HtmlCanvasElement) {
        self.canvas_elem = Some(Arc::new(canvas_elem));
    }

    pub fn register_step_logic_fn(&mut self, f: CallbackBox<WorldState>) {
        self.state.lock().unwrap().add_step_cb(f);
        self.update_step_logic_cbs();
    }

    /// Transfer callbacks from the state into the game loop.
    fn update_step_logic_cbs(&mut self) {
        self.state
            .lock()
            .unwrap()
            .transfer_callbacks(&mut self.step_logic_cbs);
    }

    pub fn setup(
        &mut self,
        program: Rc<WebGlProgram>,
        location_names: &[String],
    ) -> Result<(), String> {
        match &self.canvas_elem {
            Some(canv_arc) => {
                GameLoop::setup_input_binding(self.state.clone(), canv_arc.clone())?;
            }
            None => {
                return Err("Canvas not bound for input.".into());
            }
        }

        let program_data = self.renderer.make_program_data(program, location_names);
        let transform = Rc::new(RefCell::new(Transform::identity()));
        {
            let mut state_mg = self.state.lock().unwrap();
            state_mg.add_transform(transform.clone());
            state_mg.push_rendqueue(
                RendQueueType::Fwd,
                Rc::new(RenderItem::new(
                    program_data,
                    transform.clone(),
                    &geometry::new_square(10.),
                )),
            );
            // Unlock state.
        }
        let cb: CallbackBox<WorldState> = Rc::new(move |state| {
            square_follower(&state, transform.clone());
        });
        self.register_step_logic_fn(cb);
        Ok(())
    }

    fn setup_input_binding(
        state: Arc<Mutex<WorldState>>,
        bind_to: Arc<HtmlCanvasElement>,
    ) -> Result<(), String> {
        // Need a copy here because the listener needs its own ref.
        let bind_to_clone = bind_to.clone();
        let success = inputs::add_listener(
            bind_to.as_ref(),
            "mousemove",
            Box::new(move |evt: web_sys::Event| {
                let evt: web_sys::MouseEvent = evt.dyn_into().unwrap();
                match inputs::get_mouse_pos(bind_to_clone.as_ref(), &evt) {
                    Ok(new_input) => {
                        let mut state_mg = state.lock().unwrap();
                        state_mg.set_inputs(Some(new_input));
                    }
                    Err(_) => {
                        console::error_1(&"Mouse input failed to be captured.".into());
                    }
                }
            }),
        );
        success.map_err(|_| "Failed to add listener to canvas".into())
    }

    pub fn start(&mut self) {}

    pub fn step(&mut self) {
        self.update_step_logic_cbs();
        let mut state_mg = self.state.lock().unwrap();
        for f in self.step_logic_cbs.iter_mut() {
            f(&mut *state_mg);
        }
        self.renderer.render_all(state_mg.get_rendqueue_mut());
        state_mg.inc_frame_count();
    }
}

fn square_follower(state: &WorldState, tf: Rc<RefCell<Transform>>) {
    let mut tf = tf.borrow_mut();
    match state.get_inputs() {
        Some(inputs) => {
            let x = inputs.get_mouse_x();
            let y = inputs.get_mouse_y();
            tf.set_position(Vector3::new(x, 192.0 - y, 0.));
        }
        _ => {
            tf.set_position(Vector3::new(0., 0., 0.));
        }
    }
}
