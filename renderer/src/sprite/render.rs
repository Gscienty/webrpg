use std::collections::BTreeMap;

use crate::{
    shader,
    sprite::{render_phase::RenderPhase, utils},
    texture,
    texture_set::TextureSet,
};
use fileformat::mdx;
use shared::{
    color,
    math3::{self, FrustumContains},
};
use web_sys::WebGl2RenderingContext;

use crate::{
    Error,
    sprite::{Sprite, body::BodyRenderParams, node_matrix},
};

impl<'s> Sprite<'s> {
    pub fn render(
        &self,
        gl: &WebGl2RenderingContext,
        render_param: &RenderParams,
    ) -> Result<(), Error> {
        let model: &'s mdx::Model<'s> = unsafe { &*(self.model.as_ref().get_ref() as *const _) };

        let transform = utils::evaluate_sprite_transform(
            render_param.angle,
            render_param.origin,
            render_param.scale,
        );

        if !render_param.no_frustum_cull {
            let sphere = math3::Sphere3 {
                center: transform * self.body.bound.cube.center(),
                radius: self.body.bound.radius * render_param.scale,
            };

            if !render_param.frustum.contains(sphere)
                || !render_param
                    .frustum
                    .contains((self.body.bound.cube, transform))
            {
                return Ok(());
            }
        }

        let bone_matrices = self.bone_matrices(
            render_param.lerpfrac,
            transform,
            render_param.global_time,
            render_param.frame,
            render_param.previous_frame,
        );
        let (lights, lights_count) = self.pack_light(
            &bone_matrices,
            render_param.light,
            transform,
            render_param.global_time,
            render_param.frame,
        );

        self.body.render(
            gl,
            render_param,
            BodyRenderParams {
                model,
                bone_matrices: &bone_matrices,
                textures: &self.textures,
                transform,
                lights,
                lights_count,
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

pub struct RenderParams<'s> {
    pub origin: math3::Vector3,
    pub angle: f32,
    pub scale: f32,

    pub no_frustum_cull: bool,
    pub no_face_cull: bool,
    pub no_lighting: bool,

    pub render_phase: RenderPhase,
    pub msaa_samples: u32,
    pub lerpfrac: f32,
    pub tint: color::Color32,
    pub team: u32,

    pub global_time: u32,
    pub frame: u32,
    pub previous_frame: u32,

    pub view_projection: math3::Matrix4,
    pub texture: math3::Matrix4,
    pub light: math3::Matrix4,

    pub fog: Option<(
        math3::Vector3, /* color */
        math3::Vector2, /* range */
    )>,

    pub override_texture: Option<&'s texture::Texture>,
    pub override_slot: u32,

    pub frustum: math3::Frustum3,

    pub texture_set: &'s TextureSet,

    pub model_program: &'s shader::Program,
}
