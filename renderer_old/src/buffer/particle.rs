use std::{
    mem::{offset_of, size_of},
    ops::{Index, IndexMut},
};

use web_sys::WebGl2RenderingContext;

use super::RenderBuffer;
use crate::shader::{AttributeID, Error};

#[repr(C)]
pub struct ParticleVertex {
    pub position: shared::math3::Vector3,
    pub color: shared::color::Color32,
    pub size: f32,
    pub tail: shared::math3::Vector3,
    pub uv: [f32; 2],
    pub axis: [u8; 2],
}

#[derive(Clone, Copy)]
pub enum ParticleUvOrder {
    Billboard,
    Ribbon,
}

#[derive(Default)]
pub struct ParticleVertices(Vec<ParticleVertex>);

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

const AXIS: [[u8; 2]; 6] = [[0, 0], [255, 0], [255, 255], [255, 255], [0, 255], [0, 0]];
const UV_INDEX: [[[usize; 2]; 6]; 2] = [
    [[0, 1], [2, 1], [2, 3], [2, 3], [0, 3], [0, 1]],
    [[0, 3], [0, 1], [2, 1], [2, 1], [2, 3], [0, 3]],
];

impl ParticleVertices {
    pub(crate) fn as_bytes(&self) -> &[u8] {
        unsafe {
            std::slice::from_raw_parts(
                self.0.as_ptr().cast::<u8>(),
                self.0.len() * size_of::<ParticleVertex>(),
            )
        }
    }

    pub const fn len(&self) -> usize {
        self.0.len()
    }

    pub const fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn add_quad(
        &mut self,
        position: shared::math3::Vector3,
        tail: shared::math3::Vector3,
        uv: [f32; 4],
        color: shared::color::Color32,
        size: f32,
        order: ParticleUvOrder,
    ) {
        let order = order as usize;

        self.0.extend((0..6).map(|i| ParticleVertex {
            position,
            color,
            size,
            tail,
            uv: [uv[UV_INDEX[order][i][0]], uv[UV_INDEX[order][i][1]]],
            axis: AXIS[i],
        }));
    }

    pub fn add(
        &mut self,
        position: shared::math3::Vector3,
        tail: shared::math3::Vector3,
        uvr: shared::color::Color32,
        color: shared::color::Color32,
        size: f32,
    ) {
        let uv = [uvr.red, uvr.green, uvr.blue, uvr.alpha].map(|value| value as f32 / 255.);
        self.add_quad(position, tail, uv, color, size, ParticleUvOrder::Billboard);
    }

    pub fn create_array_object(gl: &WebGl2RenderingContext) -> Result<RenderBuffer, Error> {
        let vao = gl
            .create_vertex_array()
            .ok_or(Error::CreateVertexArrayFailed)?;
        gl.bind_vertex_array(Some(&vao));

        let vbo = gl.create_buffer().ok_or(Error::CreateBufferFailed)?;
        gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&vbo));

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
            AttributeID::ParticleAxis as u32,
            2,
            WebGl2RenderingContext::UNSIGNED_BYTE,
            true,
            stride,
            offset_of!(ParticleVertex, axis) as i32,
        );

        Ok(RenderBuffer {
            vao,
            vbo,
            ibo: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn particle_vertex_layout_matches_array_object_attributes() {
        assert_eq!(offset_of!(ParticleVertex, position), 0);
        assert_eq!(offset_of!(ParticleVertex, color), 12);
        assert_eq!(offset_of!(ParticleVertex, size), 16);
        assert_eq!(offset_of!(ParticleVertex, tail), 20);
        assert_eq!(offset_of!(ParticleVertex, uv), 32);
        assert_eq!(offset_of!(ParticleVertex, axis), 40);
        assert_eq!(size_of::<ParticleVertex>(), 44);
    }

    #[test]
    fn add_particle_decodes_uv_bytes_as_a_billboard() {
        let mut vertices = ParticleVertices::default();
        let position = shared::math3::Vector3::from((1., 2., 3.));
        let tail = shared::math3::Vector3::from((4., 5., 6.));
        let color = shared::color::Color32::white();

        vertices.add(
            position,
            tail,
            shared::color::Color32 {
                red: 0,
                green: 51,
                blue: 204,
                alpha: 255,
            },
            color,
            7.,
        );

        assert_eq!(vertices.len(), 6);
        assert_eq!(vertices[0].uv, [0., 0.2]);
        assert_eq!(vertices[2].uv, [0.8, 1.]);
        assert_eq!(vertices[0].axis, [0, 0]);
        assert_eq!(vertices[0].position, position);
        assert_eq!(vertices[0].tail, tail);
        assert_eq!(vertices[0].color, color);
        assert_eq!(vertices[0].size, 7.);
    }
}
