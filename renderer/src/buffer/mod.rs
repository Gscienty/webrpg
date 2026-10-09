mod particle;
mod ui;

use web_sys::{WebGl2RenderingContext, WebGlBuffer, WebGlVertexArrayObject};

pub use particle::{ParticleUvOrder, ParticleVertex, ParticleVertices};
pub use ui::{Vertex, Vertices};

#[derive(Debug)]
pub struct RenderBuffer {
    pub vao: WebGlVertexArrayObject,
    pub vbo: WebGlBuffer,
    pub ibo: Option<WebGlBuffer>,
    pub(crate) owns_buffers: bool,
}

impl RenderBuffer {
    pub fn release(self, gl: &WebGl2RenderingContext) {
        if self.owns_buffers {
            if let Some(ibo) = self.ibo {
                gl.delete_buffer(Some(&ibo));
            }
            gl.delete_buffer(Some(&self.vbo));
        }
        gl.delete_vertex_array(Some(&self.vao));
    }
}
