use std::collections::BTreeMap;
use strum::{EnumIter, IntoEnumIterator};

use crate::texture::Texture;

#[derive(Clone, Copy, EnumIter, PartialEq, Eq, PartialOrd, Ord)]
pub enum BackdropCorner {
    TopLeftCorner,
    TopEdge,
    TopRightCorner,
    LeftEdge,
    Center,
    RightEdge,
    BottomLeftCorner,
    BottomEdge,
    BottomRightCorner,
}

pub struct BackdropTexture {
    pub texture: Texture,
    pub color: shared::color::Color32,
}

pub struct DrawBackdrop {
    pub screen: shared::math3::Rect,
    pub bg: BackdropTexture,
    pub edge: BackdropTexture,

    pub corner_flags: u16,
    pub corner_size: f32,

    pub right: f32,
    pub top: f32,
    pub bottom: f32,
    pub left: f32,

    pub flags: u8,
}

impl DrawBackdrop {
    fn rects(
        screen: &shared::math3::Rect,
        corner_size: f32,
    ) -> BTreeMap<BackdropCorner, shared::math3::Rect> {
        let x = [0., corner_size, screen.w - corner_size, screen.w];
        let y = [0., corner_size, screen.h - corner_size, screen.h];

        BackdropCorner::iter()
            .enumerate()
            .map(|(i, corner)| {
                let column = i % 3;
                let row = i / 3;
                (
                    corner,
                    shared::math3::Rect {
                        x: screen.x + x[column],
                        y: screen.y + y[row],
                        w: x[column + 1] - x[column],
                        h: y[row + 1] - y[row],
                    },
                )
            })
            .collect()
    }

    fn edge_tile(rect: &shared::math3::Rect, edge: BackdropCorner, imagesize: f32) -> f32 {
        if imagesize <= 0. {
            return 1.;
        }

        match edge {
            BackdropCorner::LeftEdge | BackdropCorner::RightEdge => (rect.h / imagesize).ceil(),
            BackdropCorner::TopEdge | BackdropCorner::BottomEdge => (rect.w / imagesize).ceil(),
            _ => 1.,
        }
    }

    fn edge_flip(edge: BackdropCorner) -> bool {
        matches!(edge, BackdropCorner::TopEdge | BackdropCorner::BottomEdge)
    }

    fn edge_uv(tile: f32, index: u32) -> shared::math3::Rect {
        let k = 1. / 8.;
        shared::math3::Rect {
            x: index as f32 * k,
            y: 0.,
            w: k,
            h: tile,
        }
    }

    pub fn draw(&self) {
        // TODO
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rects_follow_backdrop_corner_order() {
        let rects = DrawBackdrop::rects(
            &shared::math3::Rect {
                x: 10.,
                y: 20.,
                w: 100.,
                h: 80.,
            },
            10.,
        );

        assert_eq!(
            rects[&BackdropCorner::TopLeftCorner],
            shared::math3::Rect {
                x: 10.,
                y: 20.,
                w: 10.,
                h: 10.,
            }
        );
        assert_eq!(
            rects[&BackdropCorner::Center],
            shared::math3::Rect {
                x: 20.,
                y: 30.,
                w: 80.,
                h: 60.,
            }
        );
        assert_eq!(
            rects[&BackdropCorner::BottomRightCorner],
            shared::math3::Rect {
                x: 100.,
                y: 90.,
                w: 10.,
                h: 10.,
            }
        );
    }

    #[test]
    fn edge_tile_uses_edge_length() {
        let rect = shared::math3::Rect {
            x: 0.,
            y: 0.,
            w: 25.,
            h: 35.,
        };

        assert_eq!(
            DrawBackdrop::edge_tile(&rect, BackdropCorner::LeftEdge, 10.),
            4.
        );
        assert_eq!(
            DrawBackdrop::edge_tile(&rect, BackdropCorner::TopEdge, 10.),
            3.
        );
        assert_eq!(
            DrawBackdrop::edge_tile(&rect, BackdropCorner::Center, 10.),
            1.
        );
        assert_eq!(
            DrawBackdrop::edge_tile(&rect, BackdropCorner::RightEdge, 0.),
            1.
        );
    }
}
