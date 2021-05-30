use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use na::Vector3;
use wasm_bindgen::JsCast;
use web_sys::{console, HtmlCanvasElement, WebGl2RenderingContext as Gl, WebGlProgram};

use crate::geometry;
use crate::inputs;
use crate::inputs::{Input, InputBinding};
use crate::maths_utils::*;
use crate::rendering::*;
use crate::transform::Transform;

pub enum RendQueueType {
    Fwd,
    Rev,
}

type CallbackBox<State> = Rc<dyn Fn(&mut State) -> Result<(), String>>;
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
    state: Arc<Mutex<WorldState>>,
    step_logic_cbs: Vec<CallbackBox<WorldState>>,
    renderer: Option<Box<dyn Renderer>>,
    canvas: Option<Arc<HtmlCanvasElement>>,
}

impl GameLoop {
    pub fn empty() -> Self {
        Self {
            renderer: None,
            state: Arc::new(Mutex::new(WorldState::empty())),
            step_logic_cbs: Vec::new(),
            canvas: None,
        }
    }

    /// Set the canvas object to render to.
    pub fn bind_canvas(&mut self, canvas: HtmlCanvasElement, ctx: Gl) {
        let ratio = aspect_ratio(&canvas);
        let viewport_size = 1000.;
        self.renderer = Some(Box::new(RendererOrtho3D::new(
            ctx,
            viewport_size * ratio,
            viewport_size / ratio,
            400.,
        )));
        self.canvas = Some(Arc::new(canvas));
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
        match &self.canvas {
            Some(canv_arc) => {
                GameLoop::setup_input_binding(self.state.clone(), canv_arc.clone())?;
            }
            None => {
                return Err("Canvas not bound for input.".into());
            }
        }

        let transform: Rc<_> = self.new_object();
        if let Some(renderer_ref) = self.renderer.as_ref() {
            let ctx: &_ = renderer_ref.get_ctx();
            let pd = make_program_data(
                ctx,
                program,
                location_names,
                &["a_color".into(), "a_position".into()],
            );
            let buffer_info = BufferInfo::new(ctx.clone())
                .add_buffer("a_position".into(), Box::new(geometry::new_cube(50.0)), BufferSettings::new(3, Gl::FLOAT))
                .add_buffer("a_color".into(), Box::new(get_color()), BufferSettings::new(3, Gl::UNSIGNED_BYTE).normalize());
            let vao = ctx.create_vertex_array();
            let mut state_mg = self.state.lock().unwrap();
            state_mg.push_rendqueue(
                RendQueueType::Fwd,
                Rc::new(RenderItem::new_3d(
                    pd,
                    transform.clone(),
                    buffer_info,
                    vao.unwrap(),
                )),
            );
        }
            let cb: CallbackBox<WorldState> = Rc::new(move |state| {
            mouse_follower(&state, transform.clone());
            Ok(())
        });
        self.register_step_logic_fn(cb);
        Ok(())
    }

    fn new_object(&mut self) -> Rc<RefCell<Transform>> {
        let transform = Rc::new(RefCell::new(Transform::identity()));
        let mut state_mg = self.state.lock().unwrap();
        state_mg.add_transform(transform.clone());
        transform
    }

    fn setup_input_binding(
        state: Arc<Mutex<WorldState>>,
        bind_to: Arc<HtmlCanvasElement>,
    ) -> Result<(), String> {
        // Need a copy here because the listener needs its own ref.
        let bind_to_clone = bind_to.clone();
        let r = inputs::add_listener(
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
        r.map_err(|_| "Failed to add listener to canvas".into())
    }

    pub fn start(&mut self) {}

    pub fn step(&mut self) -> Result<(), String> {
        self.update_step_logic_cbs();
        let mut state_mg = self.state.lock().unwrap();
        for f in self.step_logic_cbs.iter_mut() {
            f(&mut *state_mg)?;
        }
        if let Some(r) = &self.renderer {
            r.render_all(state_mg.get_rendqueue_mut())
        }
        state_mg.inc_frame_count();
        Ok(())
    }
}

fn mouse_follower(state: &WorldState, tf: Rc<RefCell<Transform>>) {
    let mut tf = tf.borrow_mut();
    match state.get_inputs() {
        Some(inputs) => {
            let cur_pos = tf.get_position();
            let x = inputs.get_mouse_x() * 1000. / 512.;
            let y = inputs.get_mouse_y() * 1000. / 512.;
            tf.set_position(Vector3::new(
                lerp(cur_pos[0], x, 0.5),
                lerp(cur_pos[1], y, 0.5),
                0.,
            ));
            tf.set_euler_rotation(EulerAngles3D::from_deg(y / 2., x / 2., y / 4.));
        }
        _ => {
            tf.set_position(Vector3::new(0., 0., 0.));
            tf.set_euler_rotation(EulerAngles3D::from_rad(0., 0., 0.));
        }
    }
}

fn aspect_ratio(canvas: &HtmlCanvasElement) -> f32 {
    canvas.client_width() as f32 / canvas.client_height() as f32
}

fn get_color() -> Vec<u8> {
    let v: Vec<u8> = vec![
        // left column front
        200, 70, 120, 200, 70, 120, 200, 70, 120, 200, 70, 120, 200, 70, 120, 200, 70, 120,
        // top rung front
        200, 70, 120, 200, 70, 120, 200, 70, 120, 200, 70, 120, 200, 70, 120, 200, 70, 120,
        // middle rung front
        200, 70, 120, 200, 70, 120, 200, 70, 120, 200, 70, 120, 200, 70, 120, 200, 70, 120,
        // left column back
        80, 70, 200, 80, 70, 200, 80, 70, 200, 80, 70, 200, 80, 70, 200, 80, 70, 200,
        // top rung back
        80, 70, 200, 80, 70, 200, 80, 70, 200, 80, 70, 200, 80, 70, 200, 80, 70, 200,
        // middle rung back
        80, 70, 200, 80, 70, 200, 80, 70, 200, 80, 70, 200, 80, 70, 200, 80, 70, 200,
        // top
        70, 200, 210, 70, 200, 210, 70, 200, 210, 70, 200, 210, 70, 200, 210, 70, 200, 210,
        // top rung right
        200, 200, 70, 200, 200, 70, 200, 200, 70, 200, 200, 70, 200, 200, 70, 200, 200, 70,
        // under top rung
        210, 100, 70, 210, 100, 70, 210, 100, 70, 210, 100, 70, 210, 100, 70, 210, 100, 70,
        // between top rung and middle
        210, 160, 70, 210, 160, 70, 210, 160, 70, 210, 160, 70, 210, 160, 70, 210, 160, 70,
        // top of middle rung
        70, 180, 210, 70, 180, 210, 70, 180, 210, 70, 180, 210, 70, 180, 210, 70, 180, 210,
        // right of middle rung
        100, 70, 210, 100, 70, 210, 100, 70, 210, 100, 70, 210, 100, 70, 210, 100, 70, 210,
        // bottom of middle rung.
        76, 210, 100, 76, 210, 100, 76, 210, 100, 76, 210, 100, 76, 210, 100, 76, 210, 100,
        // right of bottom
        140, 210, 80, 140, 210, 80, 140, 210, 80, 140, 210, 80, 140, 210, 80, 140, 210, 80,
        // bottom
        90, 130, 110, 90, 130, 110, 90, 130, 110, 90, 130, 110, 90, 130, 110, 90, 130, 110,
        // left side
        160, 160, 220, 160, 160, 220, 160, 160, 220, 160, 160, 220, 160, 160, 220, 160, 160, 220,
    ];
    v
    /*
    gl.buffer_data_with_opt_array_buffer(
        Gl::ARRAY_BUFFER,
        Some(js_sys::Uint8Array::from(v.as_ref()).buffer().as_ref()),
        Gl::STATIC_DRAW,
    );
    */
}
