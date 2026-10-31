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

#[derive(Clone, Copy)]
pub enum ParticleUvOrder {
    Billboard,
    Ribbon,
}

#[derive(Clone, Copy)]
pub struct ParticleQuad {
    pub point: math3::Vector3,
    pub color: color::Color32,
    pub size: f32,
    pub tail: Option<math3::Vector3>,
    pub quad_right: Option<math3::Vector3>,
    pub quad_up: Option<math3::Vector3>,
    pub u0: f32,
    pub v0: f32,
    pub u1: f32,
    pub v1: f32,
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
    pub(crate) fn as_bytes(&self) -> &[u8] {
        unsafe {
            std::slice::from_raw_parts(
                self.0.as_ptr().cast::<u8>(),
                self.0.len() * size_of::<ParticleVertex>(),
            )
        }
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn set_last_quad_texture_slot(&mut self, slot: u32) {
        for vertex in self.0.iter_mut().rev().take(6) {
            vertex.uv[2] = slot as f32;
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn add(
        &mut self,
        point: math3::Vector3,
        tail: math3::Vector3,
        quad_right: math3::Vector3,
        quad_up: math3::Vector3,
        uvr: color::Color32,
        color: color::Color32,
        size: f32,
    ) {
        let [u0, v0, u1, v1] =
            [uvr.red, uvr.green, uvr.blue, uvr.alpha].map(|value| value as f32 / 255.);
        self.add_quad(
            &ParticleQuad {
                point,
                color,
                size,
                tail: Some(tail),
                quad_right: Some(quad_right),
                quad_up: Some(quad_up),
                u0,
                v0,
                u1,
                v1,
            },
            ParticleUvOrder::Billboard,
        );
    }

    pub fn add_quad(&mut self, quad: &ParticleQuad, order: ParticleUvOrder) {
        const AXIS: [[u8; 2]; 6] = [[0, 0], [255, 0], [255, 255], [255, 255], [0, 255], [0, 0]];
        const UV_INDEX: [[[usize; 2]; 6]; 2] = [
            [[0, 1], [2, 1], [2, 3], [2, 3], [0, 3], [0, 1]],
            [[0, 3], [0, 1], [2, 1], [2, 1], [2, 3], [0, 3]],
        ];
        let uv = [quad.u0, quad.v0, quad.u1, quad.v1];
        let order = order as usize;
        let tail = quad.tail.unwrap_or_default();
        let quad_right = quad.quad_right.unwrap_or_default();
        let quad_up = quad.quad_up.unwrap_or_default();

        self.0.extend((0..6).map(|i| ParticleVertex {
            position: quad.point,
            color: quad.color,
            size: quad.size,
            tail,
            quad_right,
            quad_up,
            uv: [uv[UV_INDEX[order][i][0]], uv[UV_INDEX[order][i][1]], 0.],
            axis: AXIS[i],
        }));
    }

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
            3,
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

    #[test]
    fn add_quad_builds_two_triangles() {
        let point = math3::Vector3::from((1., 2., 3.));
        let tail = math3::Vector3::from((4., 5., 6.));
        let mut vertices = ParticleVertices::default();

        vertices.add_quad(
            &ParticleQuad {
                point,
                color: color::Color32::white(),
                size: 7.,
                tail: Some(tail),
                quad_right: None,
                quad_up: None,
                u0: 0.1,
                v0: 0.2,
                u1: 0.8,
                v1: 0.9,
            },
            ParticleUvOrder::Billboard,
        );

        assert_eq!(vertices.0.len(), 6);
        assert_eq!(vertices[0].position, point);
        assert_eq!(vertices[0].tail, tail);
        assert_eq!(vertices[0].uv, [0.1, 0.2, 0.]);
        assert_eq!(vertices[2].uv, [0.8, 0.9, 0.]);
        assert_eq!(vertices[5].axis, [0, 0]);
    }

    #[test]
    fn add_decodes_color_channels_as_billboard_uvs() {
        let point = math3::Vector3::from((1., 2., 3.));
        let tail = math3::Vector3::from((4., 5., 6.));
        let right = math3::Vector3::from((1., 0., 0.));
        let up = math3::Vector3::from((0., 1., 0.));
        let mut vertices = ParticleVertices::default();

        vertices.add(
            point,
            tail,
            right,
            up,
            color::Color32 {
                red: 0,
                green: 51,
                blue: 204,
                alpha: 255,
            },
            color::Color32::white(),
            7.,
        );

        assert_eq!(vertices.0.len(), 6);
        assert_eq!(vertices[0].uv, [0., 0.2, 0.]);
        assert_eq!(vertices[2].uv, [0.8, 1., 0.]);
        assert_eq!(vertices[0].quad_right, right);
        assert_eq!(vertices[0].quad_up, up);
    }
}
