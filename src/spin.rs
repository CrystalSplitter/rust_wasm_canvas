use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;
use std::sync::{Arc, Mutex, RwLock};

use na::Vector3;
use rand::random;
use wasm_bindgen::JsCast;
use web_sys::{Document, HtmlCanvasElement, WebGl2RenderingContext as Gl, WebGlProgram};

use crate::geometry;
use crate::inputs;
use crate::inputs::Input; // Specifically for the trait.
use crate::js_bindings;
use crate::maths_utils::*;
use crate::mesh;
use crate::rendering::*;
use crate::steppables::StepError::*;
use crate::steppables::{StepError, Steppable};
use crate::transform::Transform;
use crate::util;
use crate::world_object::{
    Material, MeshComponent, RenderComponent, WorldObject3D, WorldObject3DInit,
};
use crate::world_state::WorldState;

const FIXED_STEP_RATE: u64 = 1000 / 48;

enum LoadInStage {
    NotLoading,
    Started,
    Complete,
}

pub struct GameLoop {
    state: Arc<Mutex<WorldState>>,
    shader_pg_data: BTreeMap<String, ProgramData>,
    mesh_data_arcs: Vec<(String, Arc<Mutex<Option<String>>>)>,
    load_in_stage: LoadInStage,
    mesh_datas: BTreeMap<String, String>,
    last_step_start_t: u64,
    last_step_end_t: u64,
    last_fixed_step_end_t: u64,
    last_multistep_end_t: u64,
}

impl GameLoop {
    pub fn empty() -> Self {
        Self {
            state: Arc::new(Mutex::new(WorldState::new())),
            mesh_data_arcs: Default::default(),
            shader_pg_data: Default::default(),
            mesh_datas: Default::default(),
            load_in_stage: LoadInStage::NotLoading,
            last_step_start_t: 0,
            last_step_end_t: 0,
            last_fixed_step_end_t: 0,
            last_multistep_end_t: 0,
        }
    }

    /// Set the canvas object to render to.
    pub fn bind_canvas(&mut self, canvas: HtmlCanvasElement, ctx: Rc<Gl>) {
        let ratio = util::canvas_aspect_ratio(&canvas);
        let viewport_size = 30.;
        let world_depth = 400.;

        let mut state_mg = self.state.lock().unwrap();
        let renderer = RendererOrtho3D::new(ctx, viewport_size, viewport_size / ratio, world_depth);
        state_mg.set_renderer(renderer);
        state_mg.set_canvas(Some(Arc::new(canvas)));
    }

    pub fn add_gl_program(&mut self, name: String, pg: WebGlProgram, location_names: &[String]) {
        let s = self.state.lock().unwrap();
        let renderer = s
            .get_renderer()
            .expect("Must bind canvas before adding Gl programs");
        let locs: Vec<(_, _)> = location_names
            .into_iter()
            .map(|n: &String| {
                let n = n.to_owned();
                if n.starts_with("u_") {
                    (GlLocationType::Uniform, n)
                } else if n.starts_with("a_") {
                    (GlLocationType::Attribute, n)
                } else {
                    panic!("Uniform did not begin with expected prefix");
                }
            })
            .collect();
        let pg_data = renderer.new_program_data(&pg, &locs);
        self.shader_pg_data.insert(name, pg_data);
    }

    pub fn add_load_in_mesh(&mut self, url: String) {
        self.mesh_data_arcs.push((url, Default::default()));
    }

    pub fn load_in(&mut self) -> Result<bool, String> {
        let out = match self.load_in_stage {
            LoadInStage::NotLoading => {
                for (url, m) in self.mesh_data_arcs.iter_mut() {
                    *m = crate::mesh::wavefront_obj::get_mesh_data_from_url(url.to_owned());
                }
                self.load_in_stage = LoadInStage::Started;
                false
            }
            LoadInStage::Started => {
                let mut done = true;
                for (url, m) in self.mesh_data_arcs.iter_mut() {
                    if let Ok(lock) = m.try_lock() {
                        if let Some(data) = lock.as_ref() {
                            self.mesh_datas.insert(url.to_owned(), data.to_owned());
                        } else {
                            done = false;
                            break
                        }
                    } else {
                        done = false;
                        break
                    }
                }
                if done {
                    self.load_in_stage = LoadInStage::Complete;
                }
                done
            }
            LoadInStage::Complete => {
                true
            }
        };
        Ok(out)
    }

    /// Conduct game setup, prior to start and normal stepping behaviour.
    pub fn setup(&mut self) -> Result<(), String> {
        let mut s = self.state.lock().unwrap();
        match s.get_canvas() {
            Some(canv_arc) => {
                GameLoop::setup_input_binding(self.state.clone(), canv_arc)?;
            }
            None => {
                return Err("Canvas not bound for input.".into());
            }
        }
        s.add_scripted_component(crate::game::blocks::BlockBehavior {
            program_data: self.shader_pg_data["vertex_color"].clone(),
            mesh_data: self.mesh_datas["assets/cube.obj"].clone(),
        });
        /*
        s.add_scripted_component(TorusGen::new(
            self.shader_pg_data
                .get("vertex_color")
                .expect("No vertex_color shader")
                .clone(),
        ));
        */
        Ok(())
    }

    fn setup_input_binding(
        state: Arc<Mutex<WorldState>>,
        bind_to: Arc<HtmlCanvasElement>,
    ) -> Result<(), String> {
        let body_elem = web_sys::window()
            .ok_or("Unable to get global `window`")?
            .document()
            .ok_or("Unable to get document")?
            .body()
            .ok_or("Unable to get body element")?;

        // Need a copy here because the listener needs its own ref.
        let bind_to_clone = bind_to.clone();
        inputs::add_listener(
            &body_elem,
            "mousemove",
            Box::new(move |evt: web_sys::Event| {
                let evt: web_sys::MouseEvent = evt.dyn_into().unwrap();
                match inputs::get_mouse_pos(bind_to_clone.as_ref(), &evt) {
                    Ok(new_input) => {
                        let mut state_mg = state.lock().unwrap();
                        state_mg.set_inputs(Some(new_input));
                    }
                    Err(_) => {
                        js_bindings::error("Failed to capture mouse input");
                    }
                }
            }),
        )
        .map_err(|_| "Failed to add listener to canvas".into())
    }

    pub fn start(&mut self) -> Result<(), String> {
        let mut state_mg = self.state.lock().unwrap();
        let pre_start_cbs = state_mg.scripted_components();
        let mut error_buffer: StepError<String> = Ignore;
        for s in pre_start_cbs {
            s.borrow_mut()
                .start(&mut *state_mg)
                .or_else(|e| e.store_nonfatal(&mut error_buffer))
                .or_else(|e| e.translate())?;
        }
        error_buffer.translate()
    }

    pub fn step(&mut self) -> Result<(), String> {
        self.last_step_start_t = js_bindings::millis_now() as u64;
        let mut state_mg = self.state.lock().unwrap();
        let mut pre_step_cbs = state_mg.scripted_components();
        state_mg.run_steps(&self, &mut pre_step_cbs)?;
        self.last_step_end_t = js_bindings::millis_now() as u64;

        // Needs to be i64 due to possible integer negative overflow
        if self.last_step_end_t as i64 - self.last_fixed_step_end_t as i64 > FIXED_STEP_RATE as i64
        {
            state_mg.run_fixed_steps(&self, &mut pre_step_cbs)?;
            self.last_fixed_step_end_t = js_bindings::millis_now() as u64;
        }

        let mut late_step_cbs = state_mg.scripted_components();
        state_mg.run_late_steps(&self, &mut late_step_cbs)?;
        if let Some(r) = state_mg.get_renderer() {
            r.render_all(state_mg.get_rendqueue_mut())
        }

        state_mg.inc_frame_count();
        let new_multistep_end_t = js_bindings::millis_now() as u64;
        // Needs to be i64 due to possible integer negative overflow
        let cycle_time = new_multistep_end_t as i64 - self.last_multistep_end_t as i64;
        state_mg.delta_time = match cycle_time {
            0 => 0.5,
            ct => 1.0 / ct as f32,
        };
        self.last_multistep_end_t = new_multistep_end_t;
        Ok(())
    }

    pub fn get_mesh_data(&self, idx: usize) {
        
    }
}

fn set_rand_pos(tf: &mut Transform) {
    tf.set_position(Vector3::new(
        random::<f32>().abs() * 50.0,
        random::<f32>().abs() * 50.0,
        random::<f32>().abs() * 50.0,
    ));
}

fn spin_tf(tf: &mut Transform, delta: EulerAngles3D<f32>, delta_time: f32) {
    let mut new_rot = tf.get_euler_rotation();
    new_rot.roll = Angle::from_rad(new_rot.roll.as_rad() + delta.roll.as_rad() * delta_time);
    new_rot.pitch = Angle::from_rad(new_rot.pitch.as_rad() + delta.pitch.as_rad() * delta_time);
    new_rot.yaw = Angle::from_rad(new_rot.yaw.as_rad() + delta.yaw.as_rad() * delta_time);
    tf.set_euler_rotation(new_rot);
}

#[derive(Clone)]
struct TorusGen {
    pub program_data: ProgramData,
    tf: Vec<Rc<RefCell<Transform>>>,
}

impl TorusGen {
    pub fn new(program_data: ProgramData) -> Self {
        Self {
            program_data,
            tf: vec![],
        }
    }
}

impl Steppable<WorldState> for TorusGen {
    fn start(&mut self, s: &mut WorldState) -> Result<(), StepError<String>> {
        for _ in 0..50 {
            let new_obj = WorldObject3DInit {
                render: Some(RenderComponent {
                    gl_program_data: self.program_data.clone(),
                    renderer: s
                        .get_renderer()
                        .ok_or_else(|| Fatal("No renderer".into()))?,
                    material: Material {
                        color: (1.0, 0.6, 1.0, 1.0),
                    },
                }),
                mesh: Some(MeshComponent {
                    data: panic!("Not implemented"), //mesh::wavefront_obj::into_vertex_vec("ERROR").expect("Could not convert "),
                }),
                ..Default::default()
            }
            .init();
            let tf = new_obj.tf_rc.clone();
            set_rand_pos(&mut tf.borrow_mut());
            tf.borrow_mut().set_scale(Vector3::new(1., 1., 1.));
            self.tf.push(tf);
            s.add_world_obj(new_obj);
        }
        Ok(())
    }

    fn step(&mut self, s: &mut WorldState, _: &GameLoop) -> Result<(), StepError<String>> {
        for t in &mut self.tf {
            spin_tf(
                &mut t.borrow_mut(),
                EulerAngles3D::from_deg(27.0, 73.0, 13.0),
                s.delta_time,
            );
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
struct MouseFollower {
    pub(super) tf: Rc<RefCell<Transform>>,
}

impl Steppable<WorldState> for MouseFollower {
    fn step(&mut self, state: &mut WorldState, _: &GameLoop) -> Result<(), StepError<String>> {
        let mut tf = self.tf.borrow_mut();
        match (state.get_inputs(), state.get_canvas()) {
            (Some(inputs), Some(canvas)) => {
                let cur_pos = tf.get_position();
                let ratio = util::canvas_aspect_ratio(&canvas);
                let x = inputs.get_mouse_view_x() * 1000. * ratio;
                let y = inputs.get_mouse_view_y() * 1000. / ratio;
                tf.set_position(Vector3::new(
                    lerp(cur_pos[0], x, 0.5),
                    lerp(cur_pos[1], y, 0.5),
                    0.,
                ));
                tf.set_euler_rotation(EulerAngles3D::from_deg(
                    180.0 * state.get_frame_count() as f32 / 60.0,
                    x / 2.,
                    y / 4.,
                ));
            }
            _ => {
                tf.set_position(Vector3::zeros());
                tf.set_euler_rotation(EulerAngles3D::zeros());
            }
        }
        Ok(())
    }
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
