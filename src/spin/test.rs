bitflags! {
    pub struct ClearingBuffers: u32 {
        const DEPTH_BUFFER    = 0x00000100;
        const STENCIL_BUFFER  = 0x00000400;
        const COLOR_BUFFER    = 0x00004000;
    }
}
