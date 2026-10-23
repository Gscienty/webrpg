use std::{collections::BTreeMap, pin::Pin};

use fileformat::mdx;
use shared::math3;
use web_sys::WebGl2RenderingContext;

use crate::{
    Error,
    sprite::{
        body::{Body, BodyRenderParams},
        render_params::RenderParams,
    },
    texture_set::TextureSet,
};

pub mod body;
pub mod bound;
pub mod render_params;

mod material;
mod node_matrix;
mod render_phase;
mod utils;

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

    pub fn render(
        &self,
        gl: &WebGl2RenderingContext,
        render_param: &RenderParams,
    ) -> Result<(), Error> {
        let model: &'s mdx::Model<'s> = unsafe { &*(self.model.as_ref().get_ref() as *const _) };

        let transform = render_param.evaluate_sprite_transform();

        let bone_matrices = self.bone_matrices(
            render_param.lerpfrac,
            transform,
            render_param.global_time,
            render_param.frame,
            render_param.previous_frame,
        );

        self.body.render(
            gl,
            render_param,
            BodyRenderParams {
                model,
                bone_matrices: &bone_matrices,
                textures: &self.textures,
                transform,
            },
        )?;

        Ok(())
    }

    fn bone_matrices(
        &self,
        lerpfrac: f32,
        transform: math3::Matrix4,
        global_time: u32,
        frame1: u32,
        frame0: u32,
    ) -> BTreeMap<u32, math3::Matrix4> {
        let model_ref: &'s mdx::Model<'s> =
            unsafe { &*(self.model.as_ref().get_ref() as *const _) };

        let matrices: BTreeMap<u32, math3::Matrix4> = self
            .model
            .nodes()
            .map(|(id, node)| {
                (
                    id,
                    node_matrix::evaluate_node_matrix(
                        lerpfrac,
                        model_ref,
                        node,
                        global_time,
                        frame1,
                        frame0,
                    ),
                )
            })
            .collect();

        let mut result = BTreeMap::new();
        self.model.nodes().for_each(|(_, node)| {
            node_matrix::get_node_global_matrix(model_ref, node, transform, &matrices, &mut result);
        });

        result
    }
}
