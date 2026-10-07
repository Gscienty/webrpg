use web_sys::WebGl2RenderingContext;

use crate::{
    Error,
    buffer::{ParticleVertices, RenderBuffer, Vertices},
};

pub struct Buffers {
    pub ui: RenderBuffer,
    pub particle: RenderBuffer,
}

impl Buffers {
    pub fn init(gl: &WebGl2RenderingContext) -> Result<Self, Error> {
        let ui = Vertices::create_array_object(gl)?;
        let particle = ParticleVertices::create_array_object(gl)?;

        Ok(Self { ui, particle })
    }
}
