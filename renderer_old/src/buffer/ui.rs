use std::{
    mem::{offset_of, size_of},
    ops::{Index, IndexMut},
};

use web_sys::WebGl2RenderingContext;

use super::RenderBuffer;
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

#[derive(Default)]
pub struct Vertices(Vec<Vertex>);

impl From<Vec<Vertex>> for Vertices {
    fn from(vertices: Vec<Vertex>) -> Self {
        Self(vertices)
    }
}

impl IntoIterator for Vertices {
    type Item = Vertex;
    type IntoIter = std::vec::IntoIter<Vertex>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

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

impl Vertices {
    pub fn as_bytes(&self) -> &[u8] {
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

    pub fn bind_attributes(gl: &WebGl2RenderingContext, base: usize) {
        gl.enable_vertex_attrib_array(AttributeID::Color as u32);
        gl.enable_vertex_attrib_array(AttributeID::Position as u32);
        gl.enable_vertex_attrib_array(AttributeID::Texcoord as u32);
        gl.enable_vertex_attrib_array(AttributeID::Skin as u32);
        gl.enable_vertex_attrib_array(AttributeID::BoneWeight as u32);
        gl.enable_vertex_attrib_array(AttributeID::Normal as u32);

        let stride = size_of::<Vertex>() as i32;
        let base = base * size_of::<Vertex>();
        gl.vertex_attrib_pointer_with_i32(
            AttributeID::Color as u32,
            4,
            WebGl2RenderingContext::UNSIGNED_BYTE,
            true,
            stride,
            (base + offset_of!(Vertex, color)) as i32,
        );
        gl.vertex_attrib_pointer_with_i32(
            AttributeID::Position as u32,
            3,
            WebGl2RenderingContext::FLOAT,
            false,
            stride,
            (base + offset_of!(Vertex, position)) as i32,
        );
        gl.vertex_attrib_pointer_with_i32(
            AttributeID::Texcoord as u32,
            2,
            WebGl2RenderingContext::FLOAT,
            false,
            stride,
            (base + offset_of!(Vertex, texcoord)) as i32,
        );
        gl.vertex_attrib_pointer_with_i32(
            AttributeID::Skin as u32,
            4,
            WebGl2RenderingContext::UNSIGNED_BYTE,
            false,
            stride,
            (base + offset_of!(Vertex, skin)) as i32,
        );
        gl.vertex_attrib_pointer_with_i32(
            AttributeID::BoneWeight as u32,
            4,
            WebGl2RenderingContext::UNSIGNED_BYTE,
            true,
            stride,
            (base + offset_of!(Vertex, bone_weight)) as i32,
        );
        gl.vertex_attrib_pointer_with_i32(
            AttributeID::Normal as u32,
            3,
            WebGl2RenderingContext::FLOAT,
            false,
            stride,
            (base + offset_of!(Vertex, normal)) as i32,
        );
    }

    pub fn create_array_object(gl: &WebGl2RenderingContext) -> Result<RenderBuffer, Error> {
        let vao = gl
            .create_vertex_array()
            .ok_or(Error::CreateVertexArrayFailed)?;
        gl.bind_vertex_array(Some(&vao));

        let vbo = gl.create_buffer().ok_or(Error::CreateBufferFailed)?;
        gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&vbo));
        Self::bind_attributes(gl, 0);

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
    fn add_quad_appends_two_triangles() {
        let mut vertices = Vertices::default();
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
        let mut vertices = Vertices::default();
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
        let mut vertices = Vertices::default();
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
