use std::pin::Pin;

use fileformat::mdx;
use web_sys::WebGl2RenderingContext;

use crate::{Error, sprite::body::Body, texture_set::TextureSet};

pub mod body;
pub mod bound;

mod material;
mod node_matrix;
pub mod render_phase;
mod utils;

mod render;
mod trace;

pub use render::RenderParams;
pub use trace::TraceParam;

pub struct Sprite<'s> {
    model: Pin<Box<mdx::Model<'s>>>,

    textures: Vec<usize>,

    body: Body<'s>,
}

impl<'s> Sprite<'s> {
    pub fn new(
        gl: &WebGl2RenderingContext,
        model: Pin<Box<mdx::Model<'s>>>,

        mpq: Option<&mut webrpg_mpq::Archive>,
        texture_set: &mut TextureSet,
    ) -> Result<Self, Error> {
        let model_ref: &'s mdx::Model<'s> = unsafe { &*(model.as_ref().get_ref() as *const _) };

        let mut textures = Vec::with_capacity(model.textures.len());
        if let Some(mpq) = mpq {
            for texture in model_ref.textures.iter() {
                if texture.path.is_empty() {
                    textures.push(0);
                    continue;
                }

                let texture_id = texture_set.load_texture(gl, mpq, &texture.path)?;
                textures.push(texture_id);
            }
        }

        let body = Body::new(gl, model_ref)?;

        Ok(Self {
            model,
            textures,
            body,
        })
    }

    pub fn destroy(self, gl: &WebGl2RenderingContext) {
        self.body.destroy(gl);
    }
}
