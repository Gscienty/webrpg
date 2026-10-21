use std::{
    mem::{offset_of, size_of},
    ops::{Index, IndexMut},
};

use web_sys::WebGl2RenderingContext;

use crate::shader::AttributeID;

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
}
