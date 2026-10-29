use std::ops::{Index, IndexMut};

use shared::{color, math3};

use crate::shader::{Vertex, Vertices};

#[repr(C)]
pub struct TrailVertex {
    pub position: math3::Vector3,
    pub texcoord: math3::Vector2,
    pub color: color::Color32,
}

#[derive(Default)]
pub struct TrailVertices(Vec<TrailVertex>);

impl From<Vec<TrailVertex>> for TrailVertices {
    fn from(value: Vec<TrailVertex>) -> Self {
        Self(value)
    }
}

impl IntoIterator for TrailVertices {
    type Item = TrailVertex;
    type IntoIter = std::vec::IntoIter<TrailVertex>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl Index<usize> for TrailVertices {
    type Output = TrailVertex;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl IndexMut<usize> for TrailVertices {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.0[index]
    }
}

impl From<TrailVertices> for Vertices {
    fn from(value: TrailVertices) -> Self {
        let vertices: Vec<Vertex> = value
            .0
            .into_iter()
            .map(|item| Vertex {
                position: item.position,
                texcoord: item.texcoord,
                color: item.color,
                normal: (0., 0., 1.).into(),
                bone_weight: [255, 0, 0, 0],
                skin: Default::default(),
            })
            .collect();

        Vertices::from(vertices)
    }
}

impl TrailVertices {
    pub const fn len(&self) -> usize {
        self.0.len()
    }

    pub const fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    fn vertex(
        position: math3::Vector3,
        texcoord: math3::Vector2,
        color: color::Color32,
    ) -> TrailVertex {
        TrailVertex {
            position,
            texcoord,
            color,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn trail_quad(
        &mut self,
        a: math3::Vector3,
        b: math3::Vector3,
        c: math3::Vector3,
        d: math3::Vector3,
        uv_a: math3::Vector2,
        uv_b: math3::Vector2,
        uv_c: math3::Vector2,
        uv_d: math3::Vector2,
        ca: color::Color32,
        cb: color::Color32,
    ) {
        self.0.extend([
            Self::vertex(a, uv_a, ca),
            Self::vertex(b, uv_b, ca),
            Self::vertex(c, uv_c, cb),
            Self::vertex(a, uv_a, ca),
            Self::vertex(c, uv_c, cb),
            Self::vertex(d, uv_d, cb),
        ]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trail_quad_builds_two_triangles_with_edge_colors() {
        let [a, b, c, d] = [
            (0., 0., 0.).into(),
            (1., 0., 0.).into(),
            (1., 1., 0.).into(),
            (0., 1., 0.).into(),
        ];
        let [uv_a, uv_b, uv_c, uv_d] = [
            (0., 0.).into(),
            (1., 0.).into(),
            (1., 1.).into(),
            (0., 1.).into(),
        ];
        let ca = color::Color32::white();
        let cb = color::Color32::black();
        let mut vertices = TrailVertices::default();

        vertices.trail_quad(a, b, c, d, uv_a, uv_b, uv_c, uv_d, ca, cb);

        assert_eq!(vertices.len(), 6);
        assert_eq!(vertices[0].position, a);
        assert_eq!(vertices[1].position, b);
        assert_eq!(vertices[2].position, c);
        assert_eq!(vertices[3].position, a);
        assert_eq!(vertices[4].position, c);
        assert_eq!(vertices[5].position, d);
        assert_eq!(vertices[0].color, ca);
        assert_eq!(vertices[2].color, cb);
        assert_eq!(vertices[5].texcoord, uv_d);
    }
}
