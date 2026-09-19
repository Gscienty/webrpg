use std::cell::RefCell;

use web_sys::WebGl2RenderingContext;

use crate::{
    Error,
    buffer::{RenderBuffer, Vertices},
    render_stats::RenderStats,
    shader,
    texture::Texture,
};

pub struct Global {
    pub shader: shader::Global,
    pub ui_buffer: RenderBuffer,
    pub white_texture: Texture,
    pub drawable_width: u32,
    pub drawable_height: u32,

    pub render_stats: RefCell<RenderStats>,
}

impl Global {
    pub fn init(
        gl: &WebGl2RenderingContext,
        drawable_width: u32,
        drawable_height: u32,
    ) -> Result<Self, Error> {
        Ok(Self {
            shader: shader::Global::init(gl)?,
            ui_buffer: Vertices::default().new_array_object(gl)?,
            white_texture: Texture::white(gl)?,
            drawable_width,
            drawable_height,

            render_stats: RefCell::new(RenderStats::default()),
        })
    }
}
