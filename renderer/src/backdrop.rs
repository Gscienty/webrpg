use shared::BlendMode;
use std::collections::BTreeMap;
use strum::{EnumIter, IntoEnumIterator};
use web_sys::WebGl2RenderingContext;

use crate::{
    Error, buffer::Vertices, draw::draw_image, global::Global, shader::ShaderType, texture::Texture,
};

pub const DRAW_CLIP: u8 = 1 << 0;
pub const DRAW_WORD_WRAP: u8 = 1 << 1;
pub const DRAW_TILE: u8 = 1 << 2;
pub const DRAW_MIRRORED: u8 = 1 << 3;

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
    pub texture: Option<Texture>,
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
    fn background_rect_and_uv(&self) -> (shared::math3::Rect, shared::math3::Rect) {
        let mut background = self.screen;
        background.x += self.left;
        background.y += self.top;
        background.w -= self.left + self.right;
        background.h -= self.top + self.bottom;

        let mut uv = shared::math3::Rect::new(0., 0., 1., 1.);
        if self.flags & DRAW_MIRRORED != 0 {
            uv.x = uv.w;
            uv.w = -uv.w;
        }

        (background, uv)
    }

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

    fn edge_uv(tile: f32, index: usize) -> shared::math3::Rect {
        let k = 1. / 8.;
        shared::math3::Rect {
            x: index as f32 * k,
            y: 0.,
            w: k,
            h: tile,
        }
    }

    pub fn draw(&self, global: &Global, gl: &WebGl2RenderingContext) -> Result<(), Error> {
        if self.bg.texture.is_none() && self.edge.texture.is_none() {
            return Ok(());
        }

        let rects = Self::rects(&self.screen, self.corner_size);

        if let Some(ref texture) = self.bg.texture {
            let (background, uv) = self.background_rect_and_uv();
            let mut vertices = Vertices::default();
            vertices.add_quad(&background, &uv, self.bg.color, 0.);

            draw_image(
                global,
                gl,
                &global.ui_buffer,
                texture,
                ShaderType::UI,
                BlendMode::Blend,
                0.,
                0.,
                None,
                &vertices,
                self.flags & DRAW_TILE != 0 && (uv.w.abs() > 1. || uv.h > 1.),
            )?;
        }

        if let Some(ref texture) = self.edge.texture {
            let mut vertices = Vertices::default();
            let mut edge_repeat = false;

            for (index, corner) in BackdropCorner::iter().enumerate() {
                if (self.corner_flags & (1 << index)) == 0 {
                    continue;
                }

                let h = texture.height as f32 / 1000.;
                let tile = Self::edge_tile(&rects[&corner], corner, h);
                edge_repeat |= tile > 1.;
                let uv = Self::edge_uv(tile, index);
                let flip = Self::edge_flip(corner);

                let offset = vertices.len();

                vertices.add_quad(&rects[&corner], &uv, self.edge.color, 0.);
                if flip {
                    let texcoord = vertices[offset + 1].texcoord;
                    vertices[offset + 1].texcoord = vertices[offset + 5].texcoord;
                    vertices[offset + 5].texcoord = texcoord;
                }
            }

            if !vertices.is_empty() {
                draw_image(
                    global,
                    gl,
                    &global.ui_buffer,
                    texture,
                    ShaderType::UI,
                    BlendMode::Blend,
                    0.,
                    0.,
                    None,
                    &vertices,
                    edge_repeat,
                )?;
            }
        }

        Ok(())
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

    #[test]
    fn background_mirroring_is_independent_of_tiling() {
        fn backdrop(flags: u8) -> DrawBackdrop {
            DrawBackdrop {
                screen: shared::math3::Rect::new(10., 20., 100., 80.),
                bg: BackdropTexture {
                    texture: None,
                    color: shared::color::Color32::default(),
                },
                edge: BackdropTexture {
                    texture: None,
                    color: shared::color::Color32::default(),
                },
                corner_flags: 0,
                corner_size: 0.,
                right: 4.,
                top: 2.,
                bottom: 3.,
                left: 1.,
                flags,
            }
        }

        let (background, uv) = backdrop(DRAW_MIRRORED).background_rect_and_uv();
        assert_eq!(background, shared::math3::Rect::new(11., 22., 95., 75.));
        assert_eq!(uv, shared::math3::Rect::new(1., 0., -1., 1.));

        let (_, tiled_uv) = backdrop(DRAW_MIRRORED | DRAW_TILE).background_rect_and_uv();
        assert_eq!(tiled_uv, uv);
    }
}
