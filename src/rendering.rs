use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use js_sys::Float32Array;
use web_sys::{
    console, WebGl2RenderingContext as Gl, WebGlProgram, WebGlUniformLocation as GlULoc,
};

use crate::geometry::VertArray;
use crate::transform::Transform;

#[derive(Debug, Clone, Copy)]
enum DrawnStatus {
    NeedsDraw,
    Drawn,
}

#[derive(Debug, Clone)]
pub struct RenderableQueues {
    forward_queue: Vec<(DrawnStatus, Rc<RenderItem>)>,
    reverse_queue: Vec<(DrawnStatus, Rc<RenderItem>)>,
}

impl RenderableQueues {
    pub fn new() -> Self {
        Self {
            forward_queue: Vec::new(),
            reverse_queue: Vec::new(),
        }
    }

    pub fn push_forward_queue(&mut self, item: Rc<RenderItem>) {
        self.forward_queue.push((DrawnStatus::NeedsDraw, item));
    }

    pub fn push_reverse_queue(&mut self, item: Rc<RenderItem>) {
        self.reverse_queue.push((DrawnStatus::NeedsDraw, item));
    }
}

#[derive(Debug, Clone)]
pub struct ProgramData {
    program: Rc<WebGlProgram>,
    uniforms: HashMap<String, Rc<GlULoc>>,
}

#[derive(Debug, Clone)]
pub struct RenderItem {
    program_data: ProgramData,
    enabled: bool,
    verts: VertArray,
    vert_stride: u32,
    transform: Rc<RefCell<Transform>>,
}

impl RenderItem {
    pub fn new(
        program_data: ProgramData,
        transform: Rc<RefCell<Transform>>,
        verts: &[f32],
    ) -> RenderItem {
        RenderItem {
            program_data,
            enabled: true,
            verts: VertArray::from(verts),
            vert_stride: 2,
            transform,
        }
    }

    pub fn disable(mut self) -> Self {
        self.enabled = false;
        self
    }

    pub fn enable(mut self) -> Self {
        self.enabled = true;
        self
    }

    pub fn program_data(&self) -> &ProgramData {
        &self.program_data
    }
}

pub struct Renderer<'a> {
    pub ctx: &'a Gl,
}

enum Errors {
    FailedToGetUniformLoc { info: String },
}

impl<'a> Renderer<'a> {
    pub fn debug(&self) {
        self.ctx.buffer_data_with_opt_array_buffer(
            Gl::ARRAY_BUFFER,
            Some(&VertArray::from(vec![0., 0., 1000., 1000.].as_ref()).buffer()),
            Gl::STATIC_DRAW,
        );
        self.ctx.draw_arrays(Gl::LINES, 0, 4 / 2);
    }

    pub fn make_program_data<T>(&self, program: Rc<WebGlProgram>, uniform_names: T) -> ProgramData
    where
        T: IntoIterator,
        T::Item: ToString,
        T::Item: Eq,
        T::Item: std::hash::Hash,
    {
        let mut m: HashMap<String, _> = HashMap::new();
        for n in uniform_names {
            let n_string = n.to_string();
            let loc = Rc::new(
                self.ctx
                    .get_uniform_location(program.as_ref(), &n_string)
                    .expect("Expecting to find uniform."),
            );
            m.insert(n_string, loc);
        }
        ProgramData {
            program,
            uniforms: m,
        }
    }

    pub fn render_all(&self, queues: &RenderableQueues) {
        self.ctx.clear(Gl::COLOR_BUFFER_BIT);
        for item_tup in queues.forward_queue.iter() {
            self.draw_item(item_tup);
        }
        for item_tup in queues.reverse_queue.iter().rev() {
            self.draw_item(item_tup);
        }
    }

    fn draw_item(&self, item_tup: &(DrawnStatus, Rc<RenderItem>)) -> Result<(), Errors> {
        let (drawn_status, item) = item_tup;
        let borrowed_tf = item.transform.borrow();
        match drawn_status {
            DrawnStatus::NeedsDraw => {
                self.apply_tf(item, &borrowed_tf)?;
                self.first_time_draw_setup(item);
            }
            DrawnStatus::Drawn => {
                self.apply_tf(item, &borrowed_tf)?;
            }
        }
        self.ctx.draw_arrays(
            Gl::LINES,
            0,
            (item.verts.length() / item.vert_stride) as i32,
        );
        Ok(())
    }

    fn first_time_draw_setup(&self, item: &RenderItem) {
        self.ctx.buffer_data_with_opt_array_buffer(
            Gl::ARRAY_BUFFER,
            Some(&item.verts.buffer()),
            Gl::STATIC_DRAW,
        );
    }

    fn apply_tf(&self, item: &RenderItem, tf: &Transform) -> Result<(), Errors> {
        let u_name = "u_transformationMatrix";
        match item.program_data().uniforms.get(u_name) {
            Some(loc_rc) => {
                let loc: Option<&GlULoc> = Some(&*loc_rc);
                let values = tf.to_mat3_vec();
                self.ctx
                    .uniform_matrix3fv_with_f32_array(loc, false, &values);
                self.ctx.draw_arrays(
                    Gl::LINES,
                    0,
                    (item.verts.length() / item.vert_stride) as i32,
                );
                Ok(())
            }
            None => Err(Errors::FailedToGetUniformLoc {
                info: u_name.into(),
            }),
        }
    }
}
