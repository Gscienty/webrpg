use std::collections::BTreeMap;

use shared::math3::FrustumContains;
use web_sys::WebGl2RenderingContext;

use crate::{
    context::{Context, RenderPhase},
    draw,
    model::{
        self,
        mdx::{Geoset, Material},
    },
    shader::{LightType, ModelLight, ModelLighting, ModelState},
    texture,
    view::{View, ViewRDFlag},
};

pub const SIGHT_DISTANCE: f32 = 2000.;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum RenderEntityFlag {
    Selected = 1 << 0,
    HasLumber = 1 << 1,
    HasGold = 1 << 2,
    Hidden = 1 << 3,
    NoUbersplat = 1 << 4,
    NoFogOfWar = 1 << 5,
    NoShadow = 1 << 6,
    AttachOverhead = 1 << 7,
    NoLighting = 1 << 8,
    GroundAnchor = 1 << 9,
    FogOfWarBlocker = 1 << 10,
    PortraitLighting = 1 << 11,
    FogOfWarRevealer = 1 << 12,
    Hostile = 1 << 13,
    Hovered = 1 << 14,
    OrthoCamera = 1 << 15,
    Mounted = 1 << 16,
    HasQuest = 1 << 17,
    QuestComplete = 1 << 18,
    NotSelectable = 1 << 19,
    Neutral = 1 << 20,
    Building = 1 << 21,
    GroundConform = 1 << 22,
    GroundSurface = 1 << 23,
}

#[allow(non_upper_case_globals)]
impl RenderEntityFlag {
    pub const GroundEffect: Self = Self::OrthoCamera;
}

#[derive(Clone, Copy)]
pub struct RenderEntity<'s> {
    pub origin: shared::math3::Vector3,
    pub model: &'s model::mdx::Model,
    pub attachment_model: Option<&'s model::mdx::Model>,
    pub attachment_angles: shared::math3::Orientation,
    pub skin: Option<&'s texture::Texture>,
    pub skin_slot: u32,
    pub splat: Option<&'s texture::Texture>,
    pub name: &'s str,
    pub number: u32,
    pub generation: u32,
    pub instance_id: Option<u32>,
    pub owner: u32,
    pub team: u32,
    pub frame: u32,
    pub previous_frame: u32,
    pub camera_index: u32,
    pub flags: u32,
    pub health: u8,
    pub effect_flags: u16,
    pub effect_model: Option<&'s model::mdx::Model>,
    pub angle: f32,
    pub scale: f32,
    pub radius: f32,
    pub splat_size: f32,
    pub ground_offset: f32,
    pub ground_snapshot_z: f32,
    pub ground_snapshot_valid: bool,
    pub shadow: Option<&'s texture::Texture>,
    pub shadow_rect: shared::math3::Rect,
    pub tint: shared::color::Color32,
    pub tint_valid: bool,
    pub indicator: shared::color::Color32,
}

impl<'s> RenderEntity<'s> {
    pub fn new(model: &'s model::mdx::Model) -> Self {
        Self {
            origin: Default::default(),
            model,
            attachment_model: None,
            attachment_angles: Default::default(),
            skin: None,
            skin_slot: 0,
            splat: None,
            name: "",
            number: 0,
            generation: 0,
            instance_id: None,
            owner: 0,
            team: 0,
            frame: 0,
            previous_frame: 0,
            camera_index: 0,
            flags: 0,
            health: 0,
            effect_flags: 0,
            effect_model: None,
            angle: 0.,
            scale: 1.,
            radius: 0.,
            splat_size: 0.,
            ground_offset: 0.,
            ground_snapshot_z: 0.,
            ground_snapshot_valid: false,
            shadow: None,
            shadow_rect: Default::default(),
            tint: Default::default(),
            tint_valid: false,
            indicator: Default::default(),
        }
    }

    pub fn create_sight_matrix(
        &self,
        world_center: shared::math3::Vector2,
    ) -> shared::math3::Matrix4 {
        shared::math3::Matrix4::identity()
            .translate(
                (
                    self.origin[0] - world_center[0] - SIGHT_DISTANCE / 2.,
                    self.origin[1] - world_center[1] - SIGHT_DISTANCE / 2.,
                    0.,
                )
                    .into(),
            )
            .scale((SIGHT_DISTANCE, SIGHT_DISTANCE, SIGHT_DISTANCE).into())
    }

    pub fn caster_near_revealers(&self, revealer_list: &[&Self], num_revealers: usize) -> bool {
        let range = SIGHT_DISTANCE + self.radius + 100.;
        let range_sq = range * range;

        revealer_list.iter().take(num_revealers).any(|&revealer| {
            if std::ptr::eq(self, revealer) {
                return false;
            }

            let dx = self.origin[0] - revealer.origin[0];
            let dy = self.origin[1] - revealer.origin[1];
            dx * dx + dy * dy <= range_sq
        })
    }

    pub fn get_entity_matrix(&self) -> shared::math3::Matrix4 {
        let angle = shared::math3::Orientation::from((self.angle, 0., 0.));
        let rotation = shared::math3::Quaternion::from(&angle);

        shared::math3::Matrix4::from_rotation_translation_scale_origin(
            rotation,
            self.origin,
            shared::math3::Vector3::from((self.scale, self.scale, self.scale)),
            shared::math3::Vector3::from((0., 0., 0.)),
        )
    }

    pub fn render(&self, view: &View<'_>, context: &Context) {
        // TODO: ribbons

        let transform = self.get_entity_matrix();
        self.render_model(view, context, transform);
    }

    fn render_model(&self, view: &View<'_>, context: &Context, transform: shared::math3::Matrix4) {
        if view.rdflags & ViewRDFlag::NoFrustumCull as u32 == 0 {
            let sphere = shared::math3::Sphere3 {
                center: transform * self.model.bounds.box_.center(),
                radius: self.model.bounds.radius * self.scale,
            };
            if !view.frustum.contains(sphere)
                || !view.frustum.contains((self.model.bounds.box_, transform))
            {
                return;
            }
        }

        let mut remapped = *self;
        let entity = if self.flags & RenderEntityFlag::HasLumber as u32 != 0 {
            remapped.frame = self.model.remap_animation(remapped.frame, "Lumber");
            remapped.previous_frame = self
                .model
                .remap_animation(remapped.previous_frame, "Lumber");
            &remapped
        } else if self.flags & RenderEntityFlag::HasGold as u32 != 0 {
            remapped.frame = self.model.remap_animation(remapped.frame, "Gold");
            remapped.previous_frame = self.model.remap_animation(remapped.previous_frame, "Gold");
            &remapped
        } else {
            self
        };

        entity.render_model_inner(view, context, transform);
    }

    fn render_model_inner(
        &self,
        view: &View<'_>,
        context: &Context,
        transform: shared::math3::Matrix4,
    ) {
        let view_projection_matrix = if context.render_phase == RenderPhase::Lights {
            view.light_matrix
        } else {
            view.view_projection_matrix
        };

        let portrait_lighting = self.flags & RenderEntityFlag::PortraitLighting as u32 != 0;
        let environment = if !portrait_lighting && view.entity_light.valid {
            Some(&view.entity_light)
        } else if !portrait_lighting && view.terrain_light.valid {
            Some(&view.terrain_light)
        } else {
            None
        };

        let global_matrix = self.model.bone_matrices(
            view.lerpfrac,
            transform,
            view.time,
            self.frame,
            self.previous_frame,
        );

        let mut lighting = if let Some(environment) = environment
            && let Ok(lighting) = ModelLighting::try_from(environment)
        {
            lighting
        } else {
            ModelLighting::default()
        };
        lighting.lights.extend(self.model.collect_model_lights(
            &global_matrix,
            transform,
            view.time,
            self.frame,
        ));
        let ambient = if lighting.lights.is_empty() {
            if portrait_lighting { 0.58 } else { 0.35 }
        } else {
            if portrait_lighting { 0.22 } else { 0. }
        };
        lighting.ambient = (ambient, ambient, ambient).into();

        if lighting.lights.is_empty() {
            let directional = if portrait_lighting { 0.62 } else { 0.75 };
            lighting.lights.push(ModelLight {
                dir: (
                    -view.light_matrix[0][2],
                    -view.light_matrix[1][2],
                    -view.light_matrix[2][2],
                )
                    .into(),
                color: (directional, directional, directional).into(),
                intensity: 1.,
                typ: LightType::Direct,

                ..Default::default()
            });
        }

        let packed_lights = lighting.pack();
        let lights_count = packed_lights.len().min(8);
        let mut lights = [shared::math3::Matrix4::zero(); 8];
        lights[..lights_count].copy_from_slice(&packed_lights[..lights_count]);

        if self.flags & RenderEntityFlag::NoFogOfWar as u32 != 0 {
            context.textures.white_texture.bind(&context.gl, 2);
            context.gl.active_texture(WebGl2RenderingContext::TEXTURE0);
        }

        self.render_geosets(
            view,
            context,
            &global_matrix,
            ModelState {
                model: transform,
                normal_matrix: transform.normal(),
                unshaded: self.flags & RenderEntityFlag::NoLighting as u32 != 0,
                view_projection: view_projection_matrix,
                texture_matrix: view.texture_matrix,
                light_matrix: view.light_matrix,
                fog_enable: view.fog_enable,
                first_bone_lookup_index: 0.,
                fog_color: if view.fog_enable {
                    view.fog_color
                } else {
                    Default::default()
                },
                fog_params: if view.fog_enable {
                    (view.fog_start, view.fog_end).into()
                } else {
                    Default::default()
                },
                lights,
                lights_count: lights_count as u32,

                ..Default::default()
            },
        );

        // TODO render ribbon & particle
    }

    fn render_geosets(
        &self,
        view: &View<'_>,
        context: &Context,
        global_matrix: &BTreeMap<u32, shared::math3::Matrix4>,
        mut state: ModelState,
    ) {
        let force_unshaded = self.flags & RenderEntityFlag::NoLighting as u32 != 0;

        // TODO skin

        let color = if self.tint_valid {
            self.tint
        } else {
            shared::color::Color32::white()
        };
        let tint: shared::math3::Vector4 = (
            color.red as f32 / 255.,
            color.green as f32 / 255.,
            color.blue as f32 / 255.,
            color.alpha as f32 / 255.,
        )
            .into();
        let translucent_instance = tint[3] < 1. - f32::EPSILON;

        for blended_pass in [false, true] {
            if !context
                .render_phase
                .includes_pass(blended_pass, translucent_instance)
            {
                continue;
            }

            for geoset in &self.model.geosets {
                if !geoset.is_visible(self.model, view.time, self.frame) {
                    continue;
                }

                let Some(material) = self
                    .model
                    .materials
                    .get(geoset.property.material_id as usize)
                else {
                    continue;
                };

                if context
                    .render_phase
                    .includes_pass(false, translucent_instance)
                    && material.has_pass(false)
                {
                    self.model.render_geoset(
                        context,
                        global_matrix,
                        &mut state,
                        geoset,
                        Some(material),
                        view.fog_enable,
                        self.team,
                        self.skin,
                        self.skin_slot,
                        force_unshaded,
                        view.time,
                        self.frame,
                        tint,
                        blended_pass,
                    );
                }
            }
        }

        let mut draw_order = Vec::with_capacity(self.model.geosets.len());
        for geoset in self.model.geosets.iter() {
            if !geoset.is_visible(self.model, view.time, self.frame) {
                continue;
            }
            let material = self
                .model
                .materials
                .get(geoset.property.material_id as usize);

            if let Some(material) = material
                && !material.has_pass(true)
            {
                continue;
            }

            draw_order.push(GeosetDrawOrder {
                geoset,
                material,
                priority: if let Some(material) = material {
                    material.priority
                } else {
                    0
                },
                layer: 0,
                order: draw_order.len(),
            })
        }

        draw_order.sort_by(|lhs, rhs| {
            let left_blend = lhs
                .material
                .and_then(|material| material.layers.get(lhs.layer as usize))
                .map_or(shared::BlendMode::None as u32, |layer| {
                    layer.blend_mode as u32
                });
            let right_blend = rhs
                .material
                .and_then(|material| material.layers.get(rhs.layer as usize))
                .map_or(shared::BlendMode::None as u32, |layer| {
                    layer.blend_mode as u32
                });

            lhs.priority
                .cmp(&rhs.priority)
                .then_with(|| left_blend.cmp(&right_blend))
                .then_with(|| lhs.order.cmp(&rhs.order))
        });

        for draw in draw_order.into_iter() {
            self.model.render_geoset(
                context,
                global_matrix,
                &mut state,
                draw.geoset,
                draw.material,
                view.fog_enable,
                self.team,
                self.skin,
                self.skin_slot,
                force_unshaded,
                view.time,
                self.frame,
                tint,
                true,
            );
        }
    }
}

pub struct GeosetDrawOrder<'s> {
    pub geoset: &'s Geoset,
    pub material: Option<&'s Material>,
    pub priority: u32,
    pub layer: u32,
    pub order: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_entity_flags_match_protocol_bits() {
        assert_eq!(RenderEntityFlag::Selected as u32, 1 << 0);
        assert_eq!(RenderEntityFlag::GroundSurface as u32, 1 << 23);
        assert_eq!(
            RenderEntityFlag::OrthoCamera as u32,
            RenderEntityFlag::GroundEffect as u32
        );
    }
}
