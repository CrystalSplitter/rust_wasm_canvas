pub mod vertex_color {
    use crate::rendering::{
        BufferInfo, BufferSettings, Bufferable, ProgramData, RenderItem, Renderer,
    };
    use web_sys::WebGl2RenderingContext as Gl;

    /*
    pub fn add_vertex_color(item: RenderItem, rgba: (u16, u16, u16, u16)) -> RenderItem {
        RenderItem::builder().program_data(x.program_data).buffer_info(item.buffer_info)
    }
    pub fn new_item(renderer: impl Renderer, mesh_data: impl Bufferable + 'static, color_data: impl Bufferable + 'static) -> RenderItem {
        let info = BufferInfo::new(ctx).add_buffer(
            "a_position".into(),
            Box::new(mesh_data),
            BufferSettings {
                dim: 3,
                data_type: Gl::FLOAT,
                stride: 0,
                offset: 0,
                normalize: false,
            },
        ).add_buffer(
            "a_vertex_color".into(),
            Box::new(color_data),
            BufferSettings {
                dim: 4,
                data_type: Gl::FLOAT,
                stride: 0,
                offset: 0,
                normalize: true,
            },
        );

    }
    */
}
