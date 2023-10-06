use std::cell::RefCell;
use std::collections::BTreeMap;
use std::convert::TryInto;
use std::rc::Rc;

use na::Vector3;
use web_sys::{
    WebGl2RenderingContext as Gl, WebGlBuffer as GlBuffer, WebGlProgram,
    WebGlUniformLocation as GlULoc, WebGlVertexArrayObject as GlVao,
};

use crate::js_bindings;
use crate::transform::Transform;
use crate::{build_setter, build_setter_defaulted};

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

    pub fn get_forward_queue_len(&self) -> usize {
        self.forward_queue.len()
    }

    pub fn get_reverse_queue_len(&self) -> usize {
        self.reverse_queue.len()
    }
}

/// Type alias to indicate a value is meant to be a location.
#[derive(Debug, Clone, Copy)]
pub struct GlAttrLoc(i32);

#[derive(Debug, Clone, Copy)]
pub enum GlLocationType {
    Attribute,
    Uniform,
}

#[derive(Debug, Clone)]
pub struct ProgramData {
    uniforms: BTreeMap<String, GlULoc>,
    attributes: BTreeMap<String, GlAttrLoc>,
    vaos: BTreeMap<String, GlVao>,
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
    pub stride: i32,
    pub offset: i32,
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

#[derive(Debug, Clone)]
pub struct BufferDataBind {
    webgl_buffer: GlBuffer,
    pub settings: BufferSettings,
    data: js_sys::ArrayBuffer,
    data_len: u32,
}

impl BufferDataBind {
    /// Return the length of the original data, before conversion.
    pub fn len(&self) -> u32 {
        self.data_len
    }

    /// Return ref to the pre-buffered data
    pub fn buffer_data(&self) -> &js_sys::ArrayBuffer {
        &self.data
    }
}

#[derive(Debug, Clone)]
pub struct BufferInfo {
    ctx: Gl,
    buffers: BTreeMap<String, BufferDataBind>,
}

impl BufferInfo {
    pub fn new(ctx: Gl) -> Self {
        BufferInfo {
            ctx,
            buffers: BTreeMap::new(),
        }
    }

    /// Add a buffer
    pub fn add_buffer(
        mut self,
        name: String,
        data: impl Bufferable + 'static,
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
                data: data.as_buffer(),
                data_len: data.len(),
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
        let mut buffers = BTreeMap::new();
        for (key, data, settings) in buffer_mappings {
            let webgl_buffer = ctx
                .create_buffer()
                .unwrap_or_else(|| panic!("Unable to create buffer {}", key));
            buffers.insert(
                key,
                BufferDataBind {
                    webgl_buffer,
                    settings,
                    data: data.as_buffer(),
                    data_len: data.len(),
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
                Some(&data),
                Gl::STATIC_DRAW,
            );
        }
    }

    pub fn get_buffers(&self) -> &BTreeMap<String, BufferDataBind> {
        &self.buffers
    }
}

#[derive(Debug, Default)]
pub struct Assigned;
#[derive(Debug, Default)]
pub struct ToAssign;

#[derive(Debug)]
pub struct RenderItem {
    program_data: ProgramData,
    buffer_info: BufferInfo,
    enabled: bool,
    tf: Rc<RefCell<Transform>>,
    vao: GlVao,
    face_normal_vao: Option<GlVao>,
    draw_type: u32,
    always_redraw: bool,
}

impl RenderItem {
    pub fn builder() -> RenderItemBuilder {
        RenderItemBuilder::new()
    }
}

pub struct RenderItemBuilder {
    program_data: Option<ProgramData>,
    buffer_info: Option<BufferInfo>,
    enabled: Option<bool>,
    tf: Option<Rc<RefCell<Transform>>>,
    vao: Option<GlVao>,
    face_normal_vao: Option<GlVao>,
    draw_type: Option<u32>,
}

impl RenderItemBuilder {
    pub fn new() -> Self {
        RenderItemBuilder {
            program_data: None,
            buffer_info: None,
            enabled: Some(true),
            tf: None,
            vao: None,
            face_normal_vao: None,
            draw_type: Some(Gl::TRIANGLES),
        }
    }

    pub fn build(self) -> Option<RenderItem> {
        Some(RenderItem {
            program_data: self.program_data?,
            buffer_info: self.buffer_info?,
            enabled: self.enabled?,
            tf: self.tf?,
            vao: self.vao?,
            face_normal_vao: self.face_normal_vao,
            draw_type: self.draw_type?,
            always_redraw: false,
        })
    }

    build_setter!(enabled, bool);
    build_setter!(tf, Rc<RefCell<Transform>>);
    build_setter!(vao, GlVao);
    build_setter!(face_normal_vao, GlVao);
    build_setter!(draw_type, u32);
    build_setter!(buffer_info, BufferInfo);
    build_setter!(program_data, ProgramData);
}

pub struct RenderLineBuilder<'a, RendererT: ?Sized> {
    renderer: Option<&'a RendererT>,
    program_data: Option<ProgramData>,
    enabled: Option<bool>,
    points: Option<Vec<Vector3<f32>>>,
    colors: Vec<f32>,
    default_color: (f32, f32, f32, f32),
    vao: Option<GlVao>,
}

impl<'a, RendererT: Renderer + ?Sized> RenderLineBuilder<'a, RendererT> {
    pub fn new() -> RenderLineBuilder<'a, RendererT> {
        RenderLineBuilder {
            renderer: None,
            program_data: None,
            enabled: Some(true),
            points: None,
            vao: None,
            default_color: (1., 1., 1., 1.),
            colors: Vec::new(),
        }
    }

    pub fn build(self) -> Option<RenderItem> {
        let points = self.points?;
        let buffer_data = points.iter().fold(Vec::<f32>::new(), move |mut acc, p| {
            acc.push(p[0]);
            acc.push(p[1]);
            acc.push(p[2]);
            acc
        });
        let buffer_info = BufferInfo::new(self.renderer?.get_ctx().clone())
            .add_buffer(
                "a_position".into(),
                buffer_data,
                BufferSettings::new(3, Gl::FLOAT),
            )
            .add_buffer(
                "a_color".into(),
                self.colors,
                BufferSettings::new(4, Gl::FLOAT).normalize(),
            );
        Some(RenderItem {
            program_data: self.program_data?,
            buffer_info,
            enabled: self.enabled?,
            tf: Rc::new(RefCell::new(Transform::identity())),
            vao: self.vao?,
            face_normal_vao: None,
            draw_type: Gl::LINES,
            always_redraw: false,
        })
    }

    pub fn add_line(self, line: (Vector3<f32>, Vector3<f32>)) -> Self {
        let new_color = self.default_color.clone();
        self.add_line_with_color(line, new_color)
    }

    pub fn add_line_with_color(
        self,
        line: (Vector3<f32>, Vector3<f32>),
        color: (f32, f32, f32, f32),
    ) -> Self {
        let mut new_points: Vec<_> = self.points.unwrap_or(Vec::new());
        new_points.push(line.0);
        new_points.push(line.1);
        let mut new_colors: Vec<_> = self.colors;
        new_colors.push(color.0);
        new_colors.push(color.1);
        new_colors.push(color.2);
        new_colors.push(color.3);
        new_colors.push(color.0);
        new_colors.push(color.1);
        new_colors.push(color.2);
        new_colors.push(color.3);
        Self {
            points: Some(new_points),
            colors: new_colors,
            ..self
        }
    }

    build_setter!(enabled, bool);
    build_setter!(program_data, ProgramData);
    build_setter!(renderer, &'a RendererT);
    build_setter!(vao, GlVao);
    build_setter_defaulted!(default_color, (f32, f32, f32, f32));
}

impl RenderItem {
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
            .map(|data_bind| data_bind.len() / data_bind.settings.dim as u32)
    }

    pub fn get_buffer_info(&self) -> &BufferInfo {
        &self.buffer_info
    }

    /// Bind the vertex attribute to the passed in context.
    /// The bind settings determine how the given attribute will be bound.
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

    /// Write the stored data from each bound buffer to the passed in context.
    pub fn write_buffer_data(&self, ctx: &Gl) -> Result<(), String> {
        ctx.bind_vertex_array(Some(&self.vao));
        for (buffer_name, bind) in self.buffer_info.get_buffers().iter() {
            let attr_loc = self
                .program_data()
                .attributes
                .get(buffer_name)
                .ok_or(format!("Buffer attribute {} not found", buffer_name))?;
            ctx.bind_buffer(Gl::ARRAY_BUFFER, Some(&bind.webgl_buffer));
            ctx.buffer_data_with_opt_array_buffer(
                Gl::ARRAY_BUFFER,
                Some(&bind.data),
                Gl::STATIC_DRAW,
            );
            Self::setup_vertex_attrib(ctx, attr_loc, &bind.settings);
        }
        Ok(())
    }

    fn debug_draw_face_normals(&self, ctx: &Gl) -> Result<(), String> {
        Ok(())
    }
}

pub trait Renderer {
    /// Render all items in the passed in queues on to the WebGl2 Context
    fn render_all(&self, queues: &mut RenderableQueues);
    /// Return an immutable reference to the internal context.
    fn get_ctx(&self) -> &Gl;
    /// Conduct the first time draw set up by loading verts into the array buffer.
    fn first_time_draw_setup(&self, item: &RenderItem) -> Result<(), String> {
        item.write_buffer_data(self.get_ctx())
    }

    fn set_camera_tf(&mut self, camera_tf: Rc<RefCell<Transform>>);
    fn get_camera_tf(&self) -> Rc<RefCell<Transform>>;

    /// Create new program data from a passed in context.
    fn new_program_data(
        &self,
        program: &WebGlProgram,
        location_names: &[(GlLocationType, String)],
    ) -> ProgramData {
        let ctx = self.get_ctx();
        let mut u: BTreeMap<String, _> = Default::default();
        let mut a: BTreeMap<String, _> = Default::default();
        for (ty, name) in location_names {
            match ty {
                GlLocationType::Uniform => {
                    let loc = ctx
                        .get_uniform_location(program, name.as_ref())
                        .expect(&format!("Expecting to find uniform: {}", name));
                    u.insert(name.into(), loc);
                }
                GlLocationType::Attribute => {
                    let loc = ctx.get_attrib_location(program, name.as_ref());
                    a.insert(name.into(), GlAttrLoc(loc));
                }
            }
        }
        let v: BTreeMap<String, _> = Default::default();
        ProgramData {
            uniforms: u,
            attributes: a,
            vaos: v,
        }
    }
}

#[derive(Debug)]
enum RenderError {
    FailedToGetUniformLoc { info: String },
    NoBufferSize { info: String },
    FailedDraw { info: String },
}

#[derive(Debug, Clone)]
pub struct RendererOrtho3D {
    ctx: Rc<Gl>,
    view_height: f32,
    view_width: f32,
    projection_mat: na::Matrix4<f32>,
    camera_tf: Rc<RefCell<Transform>>,
    combined_camera_mat: RefCell<na::Matrix4<f32>>,
    tmp_mat_a: RefCell<na::Matrix4<f32>>,
    tmp_mat_b: RefCell<na::Matrix4<f32>>,
    tmp_mat_c: RefCell<na::Matrix4<f32>>,
}

impl RendererOrtho3D {
    pub fn new(ctx: Rc<Gl>, view_width: f32, view_height: f32, clip_depth: f32) -> Self {
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
            camera_tf: Rc::new(RefCell::new(Transform::identity())),
            view_height,
            view_width,
            combined_camera_mat: RefCell::new(na::Matrix4::zeros()),
            tmp_mat_a: RefCell::new(na::Matrix4::zeros()),
            tmp_mat_b: RefCell::new(na::Matrix4::zeros()),
            tmp_mat_c: RefCell::new(na::Matrix4::zeros()),
        }
    }

    fn draw_item(
        &self,
        item_tup: &(DrawnStatus, Rc<RenderItem>),
        combined_camera_mat: &na::Matrix4<f32>,
    ) -> Result<(), RenderError> {
        let (drawn_status, item) = item_tup;
        let borrowed_tf = item.tf.borrow();
        self.get_ctx().bind_vertex_array(Some(&item.vao));
        match drawn_status {
            DrawnStatus::NeedsDraw => {
                // Need to do a first time draw,
                // otherwise the transform won't
                // do anything.
                self.first_time_draw_setup(item)
                    .map_err(|e| RenderError::FailedDraw { info: e })?;
                self.apply_tf(item, &borrowed_tf, &combined_camera_mat)?;
            }
            DrawnStatus::Drawn => {
                self.apply_tf(item, &borrowed_tf, &combined_camera_mat)?;
            }
        }
        let components: i32 = item
            .get_buffer_size("a_position")
            .ok_or(RenderError::NoBufferSize {
                info: "a_position".into(),
            })?
            .try_into()
            .unwrap();
        self.ctx.draw_arrays(item.draw_type, 0, components);
        Ok(())
    }

    fn apply_tf(
        &self,
        item: &RenderItem,
        tf: &Transform,
        combined_camera_mat: &na::Matrix4<f32>,
    ) -> Result<(), RenderError> {
        const U_NAME: &str = "u_transformationMatrix";
        match item.program_data().uniforms.get(U_NAME) {
            Some(loc_rc) => {
                let loc: Option<&GlULoc> = Some(&*loc_rc);
                let mut tmp_mat_a = self.tmp_mat_a.borrow_mut();
                let mut tmp_mat_b = self.tmp_mat_c.borrow_mut();
                tf.to_mat4_into(&mut tmp_mat_a);
                combined_camera_mat.mul_to(&tmp_mat_a, &mut tmp_mat_b);
                let val_ref: Vec<f32> = tmp_mat_b.iter().copied().collect();
                self.ctx
                    .uniform_matrix4fv_with_f32_array(loc, false, &val_ref);
                Ok(())
            }
            _ => Err(RenderError::FailedToGetUniformLoc {
                info: U_NAME.into(),
            }),
        }
    }

    fn camera_tf_to_mat(&self, camera_tf: &Transform) -> na::Matrix4<f32> {
        let shift_vec = Vector3::new(self.view_width * 0.5, self.view_height * 0.5, 0.0);
        camera_tf
            .to_mat4()
            .try_inverse()
            .expect("Unreachable")
            .append_translation(&shift_vec)
    }
}

impl Renderer for RendererOrtho3D {
    fn render_all(&self, queues: &mut RenderableQueues) {
        self.ctx.clear(Gl::COLOR_BUFFER_BIT | Gl::DEPTH_BUFFER_BIT);
        let camera_tf = &self.camera_tf.borrow();
        let camera_tf_mat = self.camera_tf_to_mat(camera_tf);
        {
            let mut combined_camera_mat = self.combined_camera_mat.borrow_mut();
            self.projection_mat
                .mul_to(&camera_tf_mat, &mut combined_camera_mat);
        }
        for item_tup in queues.forward_queue.iter_mut() {
            if item_tup.1.enabled {
                match self.draw_item(item_tup, &self.combined_camera_mat.borrow()) {
                    Ok(()) => {
                        item_tup.0 = if item_tup.1.always_redraw {
                            DrawnStatus::NeedsDraw
                        } else {
                            DrawnStatus::Drawn
                        }
                    }
                    Err(e) => js_bindings::error(&format!("Error: {:?}", e)),
                }
            }
        }
        for item_tup in queues.reverse_queue.iter_mut().rev() {
            if item_tup.1.enabled {
                match self.draw_item(item_tup, &self.combined_camera_mat.borrow()) {
                    Ok(()) => {
                        item_tup.0 = if item_tup.1.always_redraw {
                            DrawnStatus::NeedsDraw
                        } else {
                            DrawnStatus::Drawn
                        }
                    }
                    Err(e) => js_bindings::error(&format!("Error: {:?}", e)),
                }
            }
        }
        // Flush, based on WebGl1 best practices.
        self.ctx.flush();
    }
    /// Return an immutable reference to the interanl context.
    fn get_ctx(&self) -> &Gl {
        &self.ctx
    }

    fn set_camera_tf(&mut self, camera_tf: Rc<RefCell<Transform>>) {
        self.camera_tf = camera_tf;
    }

    fn get_camera_tf(&self) -> Rc<RefCell<Transform>> {
        self.camera_tf.clone()
    }
}
