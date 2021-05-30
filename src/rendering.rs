use std::cell::RefCell;
use std::collections::HashMap;
use std::convert::TryInto;
use std::rc::Rc;

use web_sys::{
    WebGl2RenderingContext as Gl, WebGlBuffer as GlBuffer, WebGlProgram,
    WebGlUniformLocation as GlULoc, WebGlVertexArrayObject as GlVao,
};

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

/// Type alias to indicate a value is meant to be a location.
#[derive(Debug, Clone, Copy)]
pub struct GlAttrLoc(i32);

#[derive(Debug, Clone)]
pub struct ProgramData {
    program: Rc<WebGlProgram>,
    uniforms: HashMap<String, GlULoc>,
    attributes: HashMap<String, GlAttrLoc>,
    vaos: HashMap<String, GlVao>,
}

pub trait Bufferable: std::fmt::Debug {
    fn as_buffer(&self) -> js_sys::ArrayBuffer;
    fn len(&self) -> u32;
}

impl Bufferable for Vec<f32> {
    fn as_buffer(&self) -> js_sys::ArrayBuffer {
        js_sys::Float32Array::from(self.as_ref()).buffer()
    }

    fn len(&self) -> u32 {
        self.len() as u32
    }
}

impl Bufferable for Vec<u8> {
    fn as_buffer(&self) -> js_sys::ArrayBuffer {
        js_sys::Uint8Array::from(self.as_ref()).buffer()
    }

    fn len(&self) -> u32 {
        self.len() as u32
    }
}

#[derive(Debug, Clone)]
pub struct BufferSettings {
    pub dim: u8,
    pub data_type: u32,
    pub normalize: bool,
    stride: i32,
    offset: i32,
}

impl BufferSettings {
    pub fn new(dim: u8, data_type: u32) -> BufferSettings {
        BufferSettings {
            dim,
            data_type,
            normalize: false,
            stride: 0,
            offset: 0,
        }
    }

    pub fn normalize(mut self) -> Self {
        self.normalize = true;
        self
    }
}

#[derive(Debug)]
pub struct BufferDataBind {
    webgl_buffer: GlBuffer,
    pub settings: BufferSettings,
    data: Box<dyn Bufferable>,
}


#[derive(Debug)]
pub struct BufferInfo {
    ctx: Gl,
    buffers: HashMap<String, BufferDataBind>,
}

impl BufferInfo {
    pub fn new(ctx: Gl) -> Self {
        BufferInfo {
            ctx,
            buffers: HashMap::new(),
        }
    }

    /// Add a buffer
    pub fn add_buffer(
        mut self,
        name: String,
        data: Box<dyn Bufferable>,
        settings: BufferSettings,
    ) -> Self {
        let webgl_buffer = self
            .ctx
            .create_buffer()
            .unwrap_or_else(|| panic!("Unable to create buffer {}", name));
        self.buffers.insert(
            name,
            BufferDataBind {
                webgl_buffer,
                settings,
                data,
            },
        );
        Self {
            ctx: self.ctx,
            buffers: self.buffers,
        }
    }

    /// name order does not matter.
    pub fn from_data(
        ctx: Gl,
        buffer_mappings: Vec<(String, Box<dyn Bufferable>, BufferSettings)>,
    ) -> BufferInfo {
        let mut buffers = HashMap::with_capacity(buffer_mappings.len());
        for (key, data, settings) in buffer_mappings {
            let webgl_buffer = ctx
                .create_buffer()
                .unwrap_or_else(|| panic!("Unable to create buffer {}", key));
            buffers.insert(
                key,
                BufferDataBind {
                    webgl_buffer,
                    settings,
                    data,
                },
            );
        }
        BufferInfo { ctx, buffers }
    }

    /// Write data to the named buffers stored in this BufferInfo.
    /// Mutates the ctx.
    pub fn bind_buffer_set_data(&self) {
        for (_, data_bind) in self.buffers.iter() {
            let data = &data_bind.data;
            self.ctx
                .bind_buffer(Gl::ARRAY_BUFFER, Some(&data_bind.webgl_buffer));
            self.ctx.buffer_data_with_opt_array_buffer(
                Gl::ARRAY_BUFFER,
                Some(&data.as_buffer()),
                Gl::STATIC_DRAW,
            );
        }
    }

    pub fn get_buffers(&self) -> &HashMap<String, BufferDataBind> {
        &self.buffers
    }
}

#[derive(Debug)]
pub struct RenderItem {
    program_data: ProgramData,
    buffer_info: BufferInfo,
    enabled: bool,
    //verts: VertArray,
    vert_stride: u32,
    transform: Rc<RefCell<Transform>>,
    vao: GlVao,
}

impl RenderItem {
    pub fn new_2d(
        program_data: ProgramData,
        transform: Rc<RefCell<Transform>>,
        buffer_info: BufferInfo,
        vao: GlVao,
    ) -> RenderItem {
        RenderItem {
            program_data,
            enabled: true,
            buffer_info,
            vert_stride: 2,
            transform,
            vao,
        }
    }

    pub fn new_3d(
        program_data: ProgramData,
        transform: Rc<RefCell<Transform>>,
        buffer_info: BufferInfo,
        vao: GlVao,
    ) -> RenderItem {
        RenderItem {
            program_data,
            enabled: true,
            buffer_info,
            vert_stride: 3,
            transform,
            vao,
        }
    }

    pub fn disable(&mut self) {
        self.enabled = false;
    }

    pub fn enable(&mut self) {
        self.enabled = true;
    }

    pub fn program_data(&self) -> &ProgramData {
        &self.program_data
    }

    pub fn get_buffer_size(&self, buffer_name: &str) -> Option<u32> {
        self.buffer_info
            .buffers
            .get(buffer_name)
            .map(|data_bind| data_bind.data.len() / data_bind.settings.dim as u32)
    }

    fn setup_vertex_attrib(ctx: &Gl, attr_loc: &GlAttrLoc, bind_settings: &BufferSettings) {
        let GlAttrLoc(attr_loc) = *attr_loc;
        ctx.enable_vertex_attrib_array(attr_loc as u32);
        ctx.vertex_attrib_pointer_with_i32(
            attr_loc as u32,
            bind_settings.dim as i32,
            bind_settings.data_type,
            bind_settings.normalize,
            bind_settings.stride,
            bind_settings.offset,
        );
    }

    pub fn write_buffer_data(&self, ctx: &Gl) -> Result<(), String> {
        ctx.bind_vertex_array(Some(&self.vao));
        for (buffer_name, bind) in self.buffer_info.get_buffers().iter() {
            let attr_loc = self
                .program_data()
                .attributes
                .get(buffer_name)
                .ok_or("Buffer's attribute not found")?;
            ctx.bind_buffer(Gl::ARRAY_BUFFER, Some(&bind.webgl_buffer));
            ctx.buffer_data_with_opt_array_buffer(
                Gl::ARRAY_BUFFER,
                Some(&bind.data.as_buffer()),
                Gl::STATIC_DRAW,
            );
            Self::setup_vertex_attrib(ctx, attr_loc, &bind.settings);
        }
        Ok(())
    }
}

pub fn make_program_data<T>(
    ctx: &Gl,
    program: Rc<WebGlProgram>,
    uniform_names: T,
    attrib_names: T,
) -> ProgramData
where
    T: IntoIterator,
    T::Item: ToString,
    T::Item: Eq,
    T::Item: std::hash::Hash,
{
    let mut u: HashMap<String, _> = HashMap::new();
    for n in uniform_names {
        let n_string = n.to_string();
        let loc = ctx
            .get_uniform_location(program.as_ref(), &n_string)
            .expect("Expecting to find uniform.");
        u.insert(n_string, loc);
    }
    let mut a: HashMap<String, _> = HashMap::new();
    for n in attrib_names {
        let n_string = n.to_string();
        let loc = ctx.get_attrib_location(program.as_ref(), &n_string);
        a.insert(n_string, GlAttrLoc(loc));
    }
    let v: HashMap<String, _> = HashMap::new();
    ProgramData {
        program,
        uniforms: u,
        attributes: a,
        vaos: v,
    }
}

pub trait Renderer {
    /// Render all items in the passed in queues to the
    fn render_all(&self, queues: &mut RenderableQueues);
    /// Return an immutable reference to the internal context.
    fn get_ctx(&self) -> &Gl;
    /// Conduct the first time draw set up by loading verts into the array buffer.
    fn first_time_draw_setup(&self, item: &RenderItem) -> Result<(), String> {
        item.write_buffer_data(self.get_ctx())
    }
}

pub trait Renderer3D {}

#[derive(Debug)]
enum RenderError {
    FailedToGetUniformLoc { info: String },
    NoBufferSize { info: String },
}

#[derive(Debug, Clone)]
pub struct RendererOrtho3D {
    ctx: Gl,
    projection_mat: na::Matrix4<f32>,
}

impl RendererOrtho3D {
    pub fn new(ctx: Gl, view_width: f32, view_height: f32, clip_depth: f32) -> Self {
        ctx.enable(Gl::CULL_FACE); // Cull back faces
        ctx.enable(Gl::DEPTH_TEST); // Use depth to determine polygon draw ordering.

        Self {
            ctx,
            projection_mat: na::Matrix4::<f32>::from_columns(&[
                na::Vector4::new(2. / view_width, 0., 0., 0.),
                na::Vector4::new(0., -2. / view_height, 0., 0.),
                na::Vector4::new(0., 0., 2. / clip_depth, 0.),
                na::Vector4::new(-1., 1., 0., 1.),
            ]),
        }
    }

    fn draw_item(&self, item_tup: &(DrawnStatus, Rc<RenderItem>)) -> Result<(), RenderError> {
        let (drawn_status, item) = item_tup;
        let borrowed_tf = item.transform.borrow();
        match drawn_status {
            DrawnStatus::NeedsDraw => {
                // Need to do a first time draw,
                // otherwise the transform won't
                // do anything.
                self.first_time_draw_setup(item);
                self.apply_tf(item, &borrowed_tf)?;
            }
            DrawnStatus::Drawn => {
                self.apply_tf(item, &borrowed_tf)?;
            }
        }
        let components: i32 = item
            .get_buffer_size("a_position")
            .ok_or(RenderError::NoBufferSize {
                info: "a_position".into(),
            })?
            .try_into()
            .unwrap();
        self.ctx.draw_arrays(Gl::TRIANGLES, 0, components);
        Ok(())
    }

    fn apply_tf(&self, item: &RenderItem, tf: &Transform) -> Result<(), RenderError> {
        const U_NAME: &str = "u_transformationMatrix";
        match item.program_data().uniforms.get(U_NAME) {
            Some(loc_rc) => {
                let loc: Option<&GlULoc> = Some(&*loc_rc);
                let proj_matrix: &na::Matrix4<f32> = &self.projection_mat;
                let values = proj_matrix * tf.to_mat4();
                let val_ref: Vec<f32> = values.iter().copied().collect();
                self.ctx
                    .uniform_matrix4fv_with_f32_array(loc, false, &val_ref);
                Ok(())
            }
            _ => Err(RenderError::FailedToGetUniformLoc {
                info: U_NAME.into(),
            }),
        }
    }
}

impl Renderer for RendererOrtho3D {
    fn render_all(&self, queues: &mut RenderableQueues) {
        self.ctx.clear(Gl::COLOR_BUFFER_BIT | Gl::DEPTH_BUFFER_BIT);
        for item_tup in queues.forward_queue.iter_mut() {
            if item_tup.1.enabled && self.draw_item(item_tup).is_ok() {
                item_tup.0 = DrawnStatus::Drawn;
            }
        }
        for item_tup in queues.reverse_queue.iter_mut().rev() {
            if item_tup.1.enabled && self.draw_item(item_tup).is_ok() {
                item_tup.0 = DrawnStatus::Drawn;
            }
        }
    }
    /// Return an immutable reference to the interanl context.
    fn get_ctx(&self) -> &Gl {
        &self.ctx
    }
}
