use std::collections::BTreeMap;

use fileformat::mdx;
use shared::{color, math3};

use crate::shader::{self, TrailVertices};

pub const TRAIL_MAX_EDGES: usize = 64;
pub const TRAIL_STALE_MS: u32 = 1_000;

pub struct TrailEdge {
    pub above: math3::Vector3,
    pub below: math3::Vector3,
    pub color: color::Color32,
    pub age: f32,
}

#[derive(Default)]
pub struct Trail {
    pub edges: Vec<TrailEdge>,
    pub acc: f32,
    pub stamp: u32,
}

impl Trail {
    #[allow(clippy::too_many_arguments)]
    pub fn advance(
        &mut self,
        above: math3::Vector3,
        below: math3::Vector3,
        color: color::Color32,
        lifespan: f32,
        rate: f32,
        gravity: f32,
        frame: u32,
        delta: u32,
    ) -> usize {
        if lifespan <= 0. {
            self.edges.clear();
            self.acc = 0.;
            return 0;
        }

        let gap = frame.wrapping_sub(self.stamp);
        if self.stamp != 0 && gap > TRAIL_STALE_MS {
            self.edges.clear();
            self.acc = 0.;
        }
        let dt = if gap == 0 { 0. } else { delta as f32 / 1_000. };
        self.stamp = frame;

        for edge in &mut self.edges {
            edge.age += dt;
            edge.above = (edge.above[0], edge.above[1], edge.above[2] - gravity * dt).into();
            edge.below = (edge.below[0], edge.below[1], edge.below[2] - gravity * dt).into();
        }
        let expired = self
            .edges
            .iter()
            .take_while(|edge| edge.age >= lifespan)
            .count();
        self.edges.drain(..expired);

        if rate > 0. && dt > 0. {
            self.acc = (self.acc + rate * dt).min(2.);
            while self.acc >= 1. {
                self.acc -= 1.;
                if self.edges.len() >= TRAIL_MAX_EDGES {
                    self.edges.remove(0);
                }
                self.edges.push(TrailEdge {
                    above,
                    below,
                    color,
                    age: 0.,
                });
            }
        }

        self.edges.len()
    }

    pub fn trail_strip_verts(
        &self,
        lifespan: f32,
        columns: u32,
        rows: u32,
        slot: u32,
    ) -> TrailVertices {
        let mut out = TrailVertices::default();
        if self.edges.len() < 2 || lifespan <= 0. {
            return out;
        }

        let columns = columns.max(1);
        let rows = rows.max(1);
        let cols = columns as f32;
        let rows_f = rows as f32;
        let cell_u = (slot % columns) as f32 / cols;
        let cell_v = (slot / columns) as f32 / rows_f;
        for edges in self.edges.windows(2) {
            let old = &edges[0];
            let new = &edges[1];
            let t_old = (old.age / lifespan).min(1.);
            let t_new = (new.age / lifespan).min(1.);
            let u0 = cell_u + t_new / cols;
            let u1 = cell_u + t_old / cols;
            let bottom = cell_v + 1. / rows_f;

            out.trail_quad(
                old.above,
                old.below,
                new.below,
                new.above,
                (u1, cell_v).into(),
                (u1, bottom).into(),
                (u0, bottom).into(),
                (u0, cell_v).into(),
                old.color,
                new.color,
            );
        }

        out
    }

    pub fn emit(
        &mut self,
        ribbon: &mdx::RibbonEmitter,
        transform: math3::Matrix4,
        bone_matrices: &BTreeMap<u32, math3::Matrix4>,
        global_time: u32,
        frame: u32,
        delta: u32,
    ) -> (u32, shader::Vertices) {
        let visibility = ribbon
            .visibility
            .as_ref()
            .and_then(|visibility| visibility.get_value(global_time, frame))
            .unwrap_or(1.);
        let height_above = ribbon
            .height_above_tracks
            .as_ref()
            .and_then(|height_above| height_above.get_value(global_time, frame))
            .unwrap_or(ribbon.height_above);
        let height_below = ribbon
            .height_below_tracks
            .as_ref()
            .and_then(|height_below| height_below.get_value(global_time, frame))
            .unwrap_or(ribbon.height_below);
        let alpha = ribbon
            .alpha_tracks
            .as_ref()
            .and_then(|alpha| alpha.get_value(global_time, frame))
            .unwrap_or(ribbon.alpha);
        let color = ribbon
            .color_tracks
            .as_ref()
            .and_then(|color| color.get_value(global_time, frame))
            .unwrap_or(ribbon.color);
        let slot = ribbon
            .texture_tracks
            .as_ref()
            .and_then(|slot| slot.get_value(global_time, frame))
            .unwrap_or(ribbon.texture_slot);
        let rate = if visibility < f32::EPSILON {
            0.
        } else {
            ribbon.emission_rate as f32
        };
        let color = color::Color32 {
            red: (color[0] * 255. + 0.5).clamp(0., 255.) as u8,
            green: (color[1] * 255. + 0.5).clamp(0., 255.) as u8,
            blue: (color[2] * 255. + 0.5).clamp(0., 255.) as u8,
            alpha: (alpha * 255. + 0.5).clamp(0., 255.) as u8,
        };

        let (above, below) =
            ribbon.world_edge(transform, bone_matrices, height_above, height_below);

        self.advance(
            above,
            below,
            color,
            ribbon.lifespan,
            rate,
            ribbon.gravity,
            global_time,
            delta,
        );

        (
            self.stamp,
            self.trail_strip_verts(ribbon.lifespan, ribbon.columns, ribbon.rows, slot)
                .into(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_strip_vertices_with_atlas_uvs() {
        let trail = Trail {
            edges: vec![
                TrailEdge {
                    above: (0., 1., 0.).into(),
                    below: (0., 0., 0.).into(),
                    color: color::Color32::white(),
                    age: 1.,
                },
                TrailEdge {
                    above: (1., 1., 0.).into(),
                    below: (1., 0., 0.).into(),
                    color: color::Color32::black(),
                    age: 0.,
                },
                TrailEdge {
                    above: (2., 1., 0.).into(),
                    below: (2., 0., 0.).into(),
                    color: color::Color32::white(),
                    age: 0.,
                },
            ],
            acc: 0.,
            stamp: 0,
        };
        let vertices = trail.trail_strip_verts(2., 2, 2, 1);

        assert_eq!(vertices.len(), 12);
        assert_eq!(vertices[0].texcoord, (0.75, 0.).into());
        assert_eq!(vertices[2].texcoord, (0.5, 0.5).into());
        assert_eq!(vertices[0].color, color::Color32::white());
        assert_eq!(vertices[2].color, color::Color32::black());
    }

    #[test]
    fn advances_expires_and_emits_edges() {
        let mut trail = Trail {
            edges: vec![TrailEdge {
                above: (0., 0., 1.).into(),
                below: (0., 0., 0.).into(),
                color: color::Color32::black(),
                age: 0.95,
            }],
            acc: 0.,
            stamp: 100,
        };
        let above = (1., 0., 2.).into();
        let below = (1., 0., 1.).into();

        let alive = trail.advance(above, below, color::Color32::white(), 1., 10., 2., 200, 100);

        assert_eq!(alive, 1);
        assert_eq!(trail.edges[0].above, above);
        assert_eq!(trail.edges[0].below, below);
        assert_eq!(trail.edges[0].age, 0.);
        assert_eq!(trail.acc, 0.);
        assert_eq!(trail.stamp, 200);
    }

    #[test]
    fn clears_stale_or_disabled_trails() {
        let edge = || TrailEdge {
            above: math3::Vector3::default(),
            below: math3::Vector3::default(),
            color: color::Color32::default(),
            age: 0.,
        };
        let mut trail = Trail {
            edges: vec![edge()],
            acc: 1.,
            stamp: 1,
        };

        assert_eq!(
            trail.advance(
                math3::Vector3::default(),
                math3::Vector3::default(),
                color::Color32::default(),
                1.,
                0.,
                0.,
                TRAIL_STALE_MS + 2,
                16,
            ),
            0
        );
        assert!(trail.edges.is_empty());
        assert_eq!(trail.acc, 0.);

        trail.edges.push(edge());
        trail.acc = 1.;
        assert_eq!(
            trail.advance(
                math3::Vector3::default(),
                math3::Vector3::default(),
                color::Color32::default(),
                0.,
                1.,
                0.,
                TRAIL_STALE_MS + 3,
                16,
            ),
            0
        );
        assert!(trail.edges.is_empty());
        assert_eq!(trail.acc, 0.);
    }
}
