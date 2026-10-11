use crate::{model, texture};

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
