use web_sys::{WebGl2RenderingContext, WebGlBuffer, WebGlVertexArrayObject};

use crate::{Error, shader};

pub struct ParticleRender {
    vao: WebGlVertexArrayObject,
    vbo: WebGlBuffer,

    program: shader::Program,
}

impl ParticleRender {
    pub fn new(gl: &WebGl2RenderingContext) -> Result<Self, Error> {
        let vbo = gl.create_buffer().ok_or(Error::CreateBufferFailed)?;
        let Some(vao) = gl.create_vertex_array() else {
            gl.delete_buffer(Some(&vbo));
            return Err(Error::CreateVertexArrayFailed);
        };

        gl.bind_vertex_array(Some(&vao));
        gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&vbo));
        shader::ParticleVertices::bind_attributes(gl);
        gl.bind_vertex_array(None);

        let program = shader::builtin::PARTICLE_SHADER.compile_program(gl)?;

        Ok(Self { vao, vbo, program })
    }

    pub fn destroy(self, gl: &WebGl2RenderingContext) {
        gl.delete_vertex_array(Some(&self.vao));
        gl.delete_buffer(Some(&self.vbo));
    }
}
