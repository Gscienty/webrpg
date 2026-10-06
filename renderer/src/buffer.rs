use std::{
    mem::offset_of,
    ops::{Index, IndexMut},
};

use web_sys::{WebGl2RenderingContext, WebGlBuffer, WebGlVertexArrayObject};

use crate::shader::{AttributeID, Error};

#[repr(C)]
pub struct Vertex {
    pub position: shared::math3::Vector3,
    pub texcoord: shared::math3::Vector2,
    pub normal: shared::math3::Vector3,
    pub color: shared::color::Color32,
    pub skin: [u8; 4],
    pub bone_weight: [u8; 4],
}

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

#[derive(Clone)]
pub struct RenderBuffer {
    pub vao: WebGlVertexArrayObject,
    pub vbo: WebGlBuffer,
    pub ibo: Option<WebGlBuffer>,
}

impl RenderBuffer {
    pub fn release(self, gl: &WebGl2RenderingContext) {
        if let Some(ibo) = self.ibo {
            gl.delete_buffer(Some(&ibo));
        }
        gl.delete_buffer(Some(&self.vbo));
        gl.delete_vertex_array(Some(&self.vao));
    }
}

#[derive(Default)]
pub struct Vertices(Vec<Vertex>);

#[derive(Default)]
pub struct ParticleVertices(Vec<ParticleVertex>);

impl Index<usize> for Vertices {
    type Output = Vertex;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl IndexMut<usize> for Vertices {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.0[index]
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

impl Vertices {
    pub(crate) fn as_bytes(&self) -> &[u8] {
        unsafe {
            std::slice::from_raw_parts(
                self.0.as_ptr().cast::<u8>(),
                self.0.len() * size_of::<Vertex>(),
            )
        }
    }

    pub const fn len(&self) -> usize {
        self.0.len()
    }

    pub const fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    fn vertex(x: f32, y: f32, z: f32, u: f32, v: f32, color: shared::color::Color32) -> Vertex {
        Vertex {
            position: (x, y, z).into(),
            texcoord: (u, v).into(),
            normal: Default::default(),
            color,
            skin: Default::default(),
            bone_weight: Default::default(),
        }
    }

    pub fn add_quad(
        &mut self,
        screen: &shared::math3::Rect,
        uv: &shared::math3::Rect,
        color: shared::color::Color32,
        z: f32,
    ) {
        self.0.extend([
            Self::vertex(screen.x, screen.y, z, uv.x, uv.y, color),
            Self::vertex(screen.x + screen.w, screen.y, z, uv.x + uv.w, uv.y, color),
            Self::vertex(
                screen.x + screen.w,
                screen.y + screen.h,
                z,
                uv.x + uv.w,
                uv.y + uv.h,
                color,
            ),
            Self::vertex(screen.x, screen.y, z, uv.x, uv.y, color),
            Self::vertex(
                screen.x + screen.w,
                screen.y + screen.h,
                z,
                uv.x + uv.w,
                uv.y + uv.h,
                color,
            ),
            Self::vertex(screen.x, screen.y + screen.h, z, uv.x, uv.y + uv.h, color),
        ]);
    }

    pub(crate) fn rotate_2d(&mut self, center_x: f32, center_y: f32, radians: f32) {
        let (sin, cos) = radians.sin_cos();
        for vertex in &mut self.0 {
            let x = vertex.position[0] - center_x;
            let y = vertex.position[1] - center_y;
            vertex.position = (
                center_x + x * cos - y * sin,
                center_y + x * sin + y * cos,
                vertex.position[2],
            )
                .into();
        }
    }

    pub fn add_strip(&mut self, screen: &shared::math3::Rect, color: shared::color::Color32) {
        self.0.extend([
            Self::vertex(screen.x, screen.y, 0., 0., 0., color),
            Self::vertex(screen.x + screen.w, screen.y, 0., 0., 0., color),
            Self::vertex(screen.x + screen.w, screen.y + screen.h, 0., 0., 0., color),
            Self::vertex(screen.x, screen.y + screen.h, 0., 0., 0., color),
            Self::vertex(screen.x, screen.y, 0., 0., 0., color),
        ]);
    }

    pub fn add_wire_box(&mut self, box_: &shared::math3::Box3, color: shared::color::Color32) {
        let min = &box_.min;
        let max = &box_.max;

        self.0.extend([
            Self::vertex(min[0], min[1], min[2], 0., 0., color),
            Self::vertex(max[0], min[1], min[2], 0., 0., color),
            Self::vertex(max[0], min[1], max[2], 0., 0., color),
            Self::vertex(min[0], min[1], max[2], 0., 0., color),
            Self::vertex(min[0], min[1], min[2], 0., 0., color),
            Self::vertex(min[0], max[1], min[2], 0., 0., color),
            Self::vertex(max[0], max[1], min[2], 0., 0., color),
            Self::vertex(max[0], max[1], max[2], 0., 0., color),
            Self::vertex(min[0], max[1], max[2], 0., 0., color),
            Self::vertex(min[0], max[1], min[2], 0., 0., color),
            Self::vertex(min[0], min[1], min[2], 0., 0., color),
            Self::vertex(min[0], max[1], min[2], 0., 0., color),
            Self::vertex(max[0], min[1], min[2], 0., 0., color),
            Self::vertex(max[0], max[1], min[2], 0., 0., color),
            Self::vertex(max[0], min[1], max[2], 0., 0., color),
            Self::vertex(max[0], max[1], max[2], 0., 0., color),
            Self::vertex(min[0], min[1], max[2], 0., 0., color),
            Self::vertex(min[0], max[1], max[2], 0., 0., color),
        ]);
    }

    pub fn create_array_object(gl: &WebGl2RenderingContext) -> Result<RenderBuffer, Error> {
        let vao = gl
            .create_vertex_array()
            .ok_or(Error::CreateVertexArrayFailed)?;
        gl.bind_vertex_array(Some(&vao));

        let vbo = gl.create_buffer().ok_or(Error::CreateBufferFailed)?;
        gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&vbo));

        let stride = size_of::<Vertex>() as i32;
        for index in 0..6 {
            gl.enable_vertex_attrib_array(index);
        }
        gl.vertex_attrib_pointer_with_i32(
            0,
            4,
            WebGl2RenderingContext::UNSIGNED_BYTE,
            true,
            stride,
            offset_of!(Vertex, color) as i32,
        );
        gl.vertex_attrib_pointer_with_i32(
            1,
            3,
            WebGl2RenderingContext::FLOAT,
            false,
            stride,
            offset_of!(Vertex, position) as i32,
        );
        gl.vertex_attrib_pointer_with_i32(
            2,
            2,
            WebGl2RenderingContext::FLOAT,
            false,
            stride,
            offset_of!(Vertex, texcoord) as i32,
        );
        gl.vertex_attrib_pointer_with_i32(
            3,
            4,
            WebGl2RenderingContext::UNSIGNED_BYTE,
            false,
            stride,
            offset_of!(Vertex, skin) as i32,
        );
        gl.vertex_attrib_pointer_with_i32(
            4,
            4,
            WebGl2RenderingContext::UNSIGNED_BYTE,
            true,
            stride,
            offset_of!(Vertex, bone_weight) as i32,
        );
        gl.vertex_attrib_pointer_with_i32(
            5,
            3,
            WebGl2RenderingContext::FLOAT,
            false,
            stride,
            offset_of!(Vertex, normal) as i32,
        );

        Ok(RenderBuffer {
            vao,
            vbo,
            ibo: None,
        })
    }

    pub fn new_array_object(&self, gl: &WebGl2RenderingContext) -> Result<RenderBuffer, Error> {
        let buffer = Self::create_array_object(gl)?;

        if !self.0.is_empty() {
            gl.buffer_data_with_u8_array(
                WebGl2RenderingContext::ARRAY_BUFFER,
                self.as_bytes(),
                WebGl2RenderingContext::STATIC_DRAW,
            );
        }

        Ok(buffer)
    }

    pub fn new_indexed_array_object(
        &self,
        gl: &WebGl2RenderingContext,
        indices: &[u32],
    ) -> Result<RenderBuffer, Error> {
        let mut buffer = self.new_array_object(gl)?;
        gl.bind_vertex_array(Some(&buffer.vao));

        let ibo = gl.create_buffer().ok_or(Error::CreateBufferFailed)?;
        gl.bind_buffer(WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER, Some(&ibo));

        let bytes = unsafe {
            std::slice::from_raw_parts(indices.as_ptr().cast::<u8>(), size_of_val(indices))
        };
        gl.buffer_data_with_u8_array(
            WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER,
            bytes,
            WebGl2RenderingContext::STATIC_DRAW,
        );

        buffer.ibo = Some(ibo);
        Ok(buffer)
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

        for attribute in [
            AttributeID::Position,
            AttributeID::Color,
            AttributeID::Texcoord,
            AttributeID::ParticleSize,
            AttributeID::ParticleTail,
            AttributeID::ParticleQuadRight,
            AttributeID::ParticleQuadUp,
            AttributeID::ParticleAxis,
        ] {
            gl.enable_vertex_attrib_array(attribute as u32);
        }

        let stride = size_of::<ParticleVertex>() as i32;
        for (attribute, size, typ, normalized, offset) in [
            (
                AttributeID::Position,
                3,
                WebGl2RenderingContext::FLOAT,
                false,
                offset_of!(ParticleVertex, position),
            ),
            (
                AttributeID::Color,
                4,
                WebGl2RenderingContext::UNSIGNED_BYTE,
                true,
                offset_of!(ParticleVertex, color),
            ),
            (
                AttributeID::Texcoord,
                2,
                WebGl2RenderingContext::FLOAT,
                false,
                offset_of!(ParticleVertex, uv),
            ),
            (
                AttributeID::ParticleSize,
                1,
                WebGl2RenderingContext::FLOAT,
                false,
                offset_of!(ParticleVertex, size),
            ),
            (
                AttributeID::ParticleTail,
                3,
                WebGl2RenderingContext::FLOAT,
                false,
                offset_of!(ParticleVertex, tail),
            ),
            (
                AttributeID::ParticleAxis,
                2,
                WebGl2RenderingContext::UNSIGNED_BYTE,
                true,
                offset_of!(ParticleVertex, axis),
            ),
        ] {
            gl.vertex_attrib_pointer_with_i32(
                attribute as u32,
                size,
                typ,
                normalized,
                stride,
                offset as i32,
            );
        }

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
    fn vertex_layout_matches_array_object_attributes() {
        assert_eq!(offset_of!(Vertex, position), 0);
        assert_eq!(offset_of!(Vertex, texcoord), 12);
        assert_eq!(offset_of!(Vertex, normal), 20);
        assert_eq!(offset_of!(Vertex, color), 32);
        assert_eq!(offset_of!(Vertex, skin), 36);
        assert_eq!(offset_of!(Vertex, bone_weight), 40);
        assert_eq!(size_of::<Vertex>(), 44);
    }

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
    fn add_quad_appends_two_triangles() {
        let mut vertices = Vertices(Vec::new());
        let color = shared::color::Color32 {
            red: 1,
            green: 2,
            blue: 3,
            alpha: 4,
        };

        vertices.add_quad(
            &shared::math3::Rect {
                x: 10.,
                y: 20.,
                w: 30.,
                h: 40.,
            },
            &shared::math3::Rect {
                x: 0.1,
                y: 0.2,
                w: 0.3,
                h: 0.4,
            },
            color,
            5.,
        );

        let expected = [
            ([10., 20., 5.], [0.1, 0.2]),
            ([40., 20., 5.], [0.4, 0.2]),
            ([40., 60., 5.], [0.4, 0.6]),
            ([10., 20., 5.], [0.1, 0.2]),
            ([40., 60., 5.], [0.4, 0.6]),
            ([10., 60., 5.], [0.1, 0.6]),
        ];

        assert_eq!(vertices.0.len(), expected.len());
        for (vertex, (position, texcoord)) in vertices.0.iter().zip(expected) {
            assert_eq!(
                [vertex.position[0], vertex.position[1], vertex.position[2]],
                position
            );
            assert_eq!([vertex.texcoord[0], vertex.texcoord[1]], texcoord);
            assert_eq!(vertex.color, color);
            assert_eq!(vertex.normal, Default::default());
            assert_eq!(vertex.skin, [0; 4]);
            assert_eq!(vertex.bone_weight, [0; 4]);
        }
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

    #[test]
    fn rotates_vertices_around_a_center() {
        let mut vertices = Vertices(vec![Vertices::vertex(
            2.,
            1.,
            0.,
            0.,
            0.,
            Default::default(),
        )]);

        vertices.rotate_2d(1., 1., std::f32::consts::FRAC_PI_2);

        assert!((vertices.0[0].position[0] - 1.).abs() < f32::EPSILON);
        assert!((vertices.0[0].position[1] - 2.).abs() < f32::EPSILON);
    }

    #[test]
    fn add_strip_appends_closed_rectangle() {
        let mut vertices = Vertices(Vec::new());
        let color = shared::color::Color32 {
            red: 1,
            green: 2,
            blue: 3,
            alpha: 4,
        };

        vertices.add_strip(
            &shared::math3::Rect {
                x: 10.,
                y: 20.,
                w: 30.,
                h: 40.,
            },
            color,
        );

        let expected = [
            [10., 20., 0.],
            [40., 20., 0.],
            [40., 60., 0.],
            [10., 60., 0.],
            [10., 20., 0.],
        ];

        assert_eq!(vertices.0.len(), expected.len());
        for (vertex, position) in vertices.0.iter().zip(expected) {
            assert_eq!(
                [vertex.position[0], vertex.position[1], vertex.position[2]],
                position
            );
            assert_eq!([vertex.texcoord[0], vertex.texcoord[1]], [0.; 2]);
            assert_eq!(vertex.color, color);
        }
    }

    #[test]
    fn add_wire_box_appends_box_edges() {
        let mut vertices = Vertices(Vec::new());
        let color = shared::color::Color32 {
            red: 1,
            green: 2,
            blue: 3,
            alpha: 4,
        };
        let box_ = shared::math3::Box3::from(((1., 2., 3.).into(), (4., 5., 6.).into()));

        vertices.add_wire_box(&box_, color);

        let expected = [
            [1., 2., 3.],
            [4., 2., 3.],
            [4., 2., 6.],
            [1., 2., 6.],
            [1., 2., 3.],
            [1., 5., 3.],
            [4., 5., 3.],
            [4., 5., 6.],
            [1., 5., 6.],
            [1., 5., 3.],
            [1., 2., 3.],
            [1., 5., 3.],
            [4., 2., 3.],
            [4., 5., 3.],
            [4., 2., 6.],
            [4., 5., 6.],
            [1., 2., 6.],
            [1., 5., 6.],
        ];

        assert_eq!(vertices.0.len(), expected.len());
        for (vertex, position) in vertices.0.iter().zip(expected) {
            assert_eq!(
                [vertex.position[0], vertex.position[1], vertex.position[2]],
                position
            );
            assert_eq!([vertex.texcoord[0], vertex.texcoord[1]], [0.; 2]);
            assert_eq!(vertex.color, color);
        }
    }
}
