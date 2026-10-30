use std::{
    mem::{offset_of, size_of},
    ops::{Index, IndexMut},
};

use shared::{color, math3};
use web_sys::WebGl2RenderingContext;

use crate::shader::AttributeID;

#[repr(C)]
pub struct ParticleVertex {
    pub position: math3::Vector3,
    pub color: color::Color32,
    pub size: f32,
    pub tail: math3::Vector3,
    pub quad_right: math3::Vector3,
    pub quad_up: math3::Vector3,
    pub uv: [f32; 3],
    pub axis: [u8; 2],
}

#[derive(Default)]
pub struct ParticleVertices(Vec<ParticleVertex>);

impl From<Vec<ParticleVertex>> for ParticleVertices {
    fn from(vertices: Vec<ParticleVertex>) -> Self {
        Self(vertices)
    }
}

impl Index<usize> for ParticleVertices {
    type Output = ParticleVertex;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl IndexMut<usize> for ParticleVertices {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.0[index]
    }
}

impl ParticleVertices {
    pub fn bind_attributes(gl: &WebGl2RenderingContext) {
        gl.enable_vertex_attrib_array(AttributeID::Position as u32);
        gl.enable_vertex_attrib_array(AttributeID::Color as u32);
        gl.enable_vertex_attrib_array(AttributeID::Texcoord as u32);
        gl.enable_vertex_attrib_array(AttributeID::ParticleSize as u32);
        gl.enable_vertex_attrib_array(AttributeID::ParticleTail as u32);
        gl.enable_vertex_attrib_array(AttributeID::ParticleQuadRight as u32);
        gl.enable_vertex_attrib_array(AttributeID::ParticleQuadUp as u32);
        gl.enable_vertex_attrib_array(AttributeID::ParticleAxis as u32);

        let stride = size_of::<ParticleVertex>() as i32;
        gl.vertex_attrib_pointer_with_i32(
            AttributeID::Position as u32,
            3,
            WebGl2RenderingContext::FLOAT,
            false,
            stride,
            offset_of!(ParticleVertex, position) as i32,
        );
        gl.vertex_attrib_pointer_with_i32(
            AttributeID::Color as u32,
            4,
            WebGl2RenderingContext::UNSIGNED_BYTE,
            true,
            stride,
            offset_of!(ParticleVertex, color) as i32,
        );
        gl.vertex_attrib_pointer_with_i32(
            AttributeID::Texcoord as u32,
            2,
            WebGl2RenderingContext::FLOAT,
            false,
            stride,
            offset_of!(ParticleVertex, uv) as i32,
        );
        gl.vertex_attrib_pointer_with_i32(
            AttributeID::ParticleSize as u32,
            1,
            WebGl2RenderingContext::FLOAT,
            false,
            stride,
            offset_of!(ParticleVertex, size) as i32,
        );
        gl.vertex_attrib_pointer_with_i32(
            AttributeID::ParticleTail as u32,
            3,
            WebGl2RenderingContext::FLOAT,
            false,
            stride,
            offset_of!(ParticleVertex, tail) as i32,
        );
        gl.vertex_attrib_pointer_with_i32(
            AttributeID::ParticleQuadRight as u32,
            3,
            WebGl2RenderingContext::FLOAT,
            false,
            stride,
            offset_of!(ParticleVertex, quad_right) as i32,
        );
        gl.vertex_attrib_pointer_with_i32(
            AttributeID::ParticleQuadUp as u32,
            3,
            WebGl2RenderingContext::FLOAT,
            false,
            stride,
            offset_of!(ParticleVertex, quad_up) as i32,
        );
        gl.vertex_attrib_pointer_with_i32(
            AttributeID::ParticleAxis as u32,
            2,
            WebGl2RenderingContext::UNSIGNED_BYTE,
            true,
            stride,
            offset_of!(ParticleVertex, axis) as i32,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn particle_vertex_layout_matches_attributes() {
        assert_eq!(offset_of!(ParticleVertex, position), 0);
        assert_eq!(offset_of!(ParticleVertex, color), 12);
        assert_eq!(offset_of!(ParticleVertex, size), 16);
        assert_eq!(offset_of!(ParticleVertex, tail), 20);
        assert_eq!(offset_of!(ParticleVertex, quad_right), 32);
        assert_eq!(offset_of!(ParticleVertex, quad_up), 44);
        assert_eq!(offset_of!(ParticleVertex, uv), 56);
        assert_eq!(offset_of!(ParticleVertex, axis), 68);
        assert_eq!(size_of::<ParticleVertex>(), 72);
    }
}
