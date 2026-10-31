use std::collections::BTreeMap;

use shared::math3;
use web_sys::{WebGl2RenderingContext, WebGlBuffer, WebGlVertexArrayObject};

use crate::{
    Error, ribbon, shader,
    sprite::{RenderParams, RenderState, Sprite, material, utils},
};

pub(super) struct Ribbon {
    vertex_buffer: WebGlBuffer,
    vertex_array: WebGlVertexArrayObject,
}

impl Ribbon {
    pub fn new(gl: &WebGl2RenderingContext) -> Result<Self, Error> {
        let vertex_buffer = gl.create_buffer().ok_or(Error::CreateBufferFailed)?;
        let Some(vertex_array) = gl.create_vertex_array() else {
            gl.delete_buffer(Some(&vertex_buffer));
            return Err(Error::CreateVertexArrayFailed);
        };

        gl.bind_vertex_array(Some(&vertex_array));
        gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&vertex_buffer));
        shader::Vertices::bind_attributes(gl, 0);
        gl.bind_vertex_array(None);

        Ok(Self {
            vertex_buffer,
            vertex_array,
        })
    }

    pub fn destroy(self, gl: &WebGl2RenderingContext) {
        gl.delete_vertex_array(Some(&self.vertex_array));
        gl.delete_buffer(Some(&self.vertex_buffer));
    }
}

impl<'s> Sprite<'s> {
    pub fn create_ribbon_instance(&self, team: u32) -> ribbon::Instance {
        let mut instance = ribbon::Instance::new(team);
        instance
            .trails
            .resize_with(self.model.ribbons.len(), ribbon::Trail::default);
        instance
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn render_ribbon(
        &self,
        gl: &WebGl2RenderingContext,
        render_param: &RenderParams,
        render_state: &mut RenderState,
        transform: math3::Matrix4,
        bone_matrices: &BTreeMap<u32, math3::Matrix4>,
        lights: [math3::Matrix4; 8],
        lights_count: u32,
    ) -> Result<(), Error> {
        if self.model.ribbons.is_empty() {
            return Ok(());
        }
        if !self.selectable && render_param.frame == render_param.previous_frame {
            return Ok(());
        }

        for (index, ribbon) in self.model.ribbons.iter().enumerate() {
            let Some(vertices) = render_state.ribbon.emit(
                index,
                ribbon,
                transform,
                bone_matrices,
                render_param.global_time,
                render_param.frame,
                render_param.delta,
            ) else {
                continue;
            };
            if vertices.is_empty() {
                continue;
            }
            self.draw_ribbon(gl, render_param, ribbon, &vertices, lights, lights_count)?;
        }

        Ok(())
    }

    pub(super) fn draw_ribbon(
        &self,
        gl: &WebGl2RenderingContext,
        render_param: &RenderParams,
        ribbon: &fileformat::mdx::RibbonEmitter,
        vertices: &shader::Vertices,
        lights: [math3::Matrix4; 8],
        lights_count: u32,
    ) -> Result<(), Error> {
        let Some(material) = self.model.materials.get(ribbon.material_id as usize) else {
            return Ok(());
        };

        gl.bind_vertex_array(Some(&self.ribbon.vertex_array));
        gl.bind_buffer(
            WebGl2RenderingContext::ARRAY_BUFFER,
            Some(&self.ribbon.vertex_buffer),
        );
        gl.buffer_data_with_u8_array(
            WebGl2RenderingContext::ARRAY_BUFFER,
            vertices.as_bytes(),
            WebGl2RenderingContext::STREAM_DRAW,
        );

        let identity = math3::Matrix4::identity();
        for (layer_id, layer) in material.layers.iter().enumerate() {
            if !render_param
                .render_phase
                .includes_pass(layer.is_blended(), false)
            {
                continue;
            }

            let mut state = shader::ModelState {
                bones_count: 1,
                view_projection: render_param.view_projection,
                light_matrix: render_param.light,
                texture_matrix: render_param.texture,
                lights,
                lights_count,
                model: identity,
                normal_matrix: identity.normal(),
                layer_alpha: 1.,
                geoset_color: (1., 1., 1., 1.).into(),
                uv_matrix: layer
                    .bind_texture_animation(render_param.global_time, render_param.frame),
                unshaded: render_param.no_lighting
                    || layer.flags & fileformat::mdx::GeoFlag::Unshared as u32 != 0,
                fog_enable: render_param.fog.is_some()
                    && layer.flags & fileformat::mdx::GeoFlag::Unfogged as u32 == 0
                    && matches!(
                        layer.blend_mode,
                        shared::BlendMode::None
                            | shared::BlendMode::AlphaKey
                            | shared::BlendMode::Blend
                    ),
                fog_color: render_param.fog.map(|(color, _)| color).unwrap_or_default(),
                fog_params: render_param.fog.map(|(_, range)| range).unwrap_or_default(),
                ..Default::default()
            };
            state.bones[0] = identity;

            gl.enable(WebGl2RenderingContext::DEPTH_TEST);
            gl.disable(WebGl2RenderingContext::CULL_FACE);
            state.alpha_key = false;
            if !material::layer_set_blended(
                gl,
                layer_id,
                layer,
                render_param.render_phase,
                render_param.msaa_samples,
                &mut state,
            ) {
                continue;
            }
            material::layer_apply_flags(gl, layer);
            if render_param.no_depth_test {
                gl.disable(WebGl2RenderingContext::DEPTH_TEST);
            }
            if render_param.no_depth_write {
                gl.depth_mask(false);
            }

            let model_texture_id =
                layer.evaluate_texture_id(render_param.global_time, render_param.frame);
            let Some(model_texture) = self.model.textures.get(model_texture_id) else {
                continue;
            };
            let Some(&texture_id) = self.textures.get(model_texture_id) else {
                continue;
            };
            let Some(texture) = render_param.texture_set.get_texture(
                texture_id,
                render_param.team,
                model_texture.replaceable_id,
                render_param.override_texture,
                render_param.override_slot,
            ) else {
                continue;
            };

            texture.bind(gl, 0);
            render_param.model_program.apply(gl, &state)?;
            gl.draw_arrays(WebGl2RenderingContext::TRIANGLES, 0, vertices.len() as i32);
        }

        gl.enable(WebGl2RenderingContext::CULL_FACE);
        gl.depth_mask(true);
        utils::set_alpha_key_state(
            gl,
            render_param.render_phase,
            render_param.msaa_samples,
            false,
        );

        Ok(())
    }
}
