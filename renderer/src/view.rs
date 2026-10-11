use shared::math3::Line3Intersect;

use crate::{
    context::Context,
    model,
    render_entity::{RenderEntity, RenderEntityFlag},
    shader::EnvironLight,
    terrian_mask::TerrainMask,
};

#[repr(u32)]
pub enum ViewRDFlag {
    NoFog = 0x01,
    NoFogMask = 0x02,
    NoWorldModel = 0x04,
    NoFrustumCull = 0x08,
    NoParticles = 0x10,
    UseEntityCamera = 0x20,
    IsolatedParticles = 0x40,
    UISprite = 0x80,
}

#[repr(u32)]
pub enum ViewFlag {
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
    FowBlocker = 1 << 10,
    PortraitLighting = 1 << 11,
    FowRevealer = 1 << 12,
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
    Unit = 1 << 24,
    Allied = 1 << 25,
    SelectionCircleOnWater = 1 << 26,
    GroundSurfaceSupport = 1 << 27,
}

#[allow(non_upper_case_globals)]
impl ViewFlag {
    pub const GroundEffect: Self = Self::OrthoCamera;
}

#[derive(Default, Clone, Copy)]
pub struct ViewCamera {
    pub origin: shared::math3::Vector3,
    pub eye: shared::math3::Vector3,
    pub view_angles: shared::math3::Vector3,
    pub distance: f32,
    pub fov: f32,
    pub znear: f32,
    pub zfar: f32,
}

#[derive(Default)]
pub struct RenderSplatRect {
    pub mins: shared::math3::Vector2,
    pub maxs: shared::math3::Vector2,
    pub color: shared::color::Color32,
}

#[derive(Default)]
pub struct WeatherEffect {
    pub handle: u32,
    pub effect_id: u32,
    pub bounds: shared::math3::Box2,
    pub enabled: u32,
}

#[derive(Default)]
pub struct LightningEffects {
    pub handle: u32,
    pub effect_id: u32,
    pub source: shared::math3::Vector3,
    pub target: shared::math3::Vector3,
    pub color: shared::color::Color32,
    pub start_time: u32,
    pub end_time: u32,
}

#[derive(Default)]
pub struct View<'s> {
    pub camera_state: [ViewCamera; 2],
    pub target: shared::math3::Vector3,
    pub viewport: shared::math3::Rect,
    pub scissor: shared::math3::Rect,
    pub time: u32,
    pub delta_time: u32,
    pub lerpfrac: f32,
    pub entities: Vec<RenderEntity<'s>>,
    pub decals: Vec<u32>,
    pub splat_rects: Vec<RenderSplatRect>,
    pub weather_effects: Vec<WeatherEffect>,
    pub lightning_effects: Vec<LightningEffects>,
    pub view_projection_matrix: shared::math3::Matrix4,
    pub light_matrix: shared::math3::Matrix4,
    pub texture_matrix: shared::math3::Matrix4,
    pub terrain_light_model: Option<&'s model::mdx::Model>,
    pub entity_light_model: Option<&'s model::mdx::Model>,
    pub sky_model: Option<&'s model::mdx::Model>,
    pub environment_phase: f32,
    pub terrain_light: EnvironLight,
    pub entity_light: EnvironLight,
    pub player: usize,
    pub rdflags: u32,
    pub hover_entity: usize,
    pub fow_width: u32,
    pub fow_height: u32,
    pub fow_generation: u32,
    // TODO: fow_data
    pub terrian_mask: TerrainMask,
    pub frustum: shared::math3::Frustum3,
    pub fog_enable: bool,
    pub fog_start: f32,
    pub fog_end: f32,
    pub fog_color: shared::math3::Vector3,
}

impl View<'_> {
    pub fn render(&self, context: &Context) {}

    pub fn point_to_view_space(&self, context: &Context, x: f32, y: f32) -> shared::math3::Vector2 {
        let mut viewport = self.viewport;
        if viewport.w <= 0. || viewport.h <= 0. {
            viewport = shared::math3::Rect::new(0., 0., 1., 1.);
        }

        let window_width = context.window_width as f32;
        let window_height = context.window_height as f32;
        let left = viewport.x * window_width;
        let top = (1. - viewport.y - viewport.h) * window_height;
        let width = viewport.w * window_width;
        let height = viewport.h * window_height;

        (
            ((x - left) / width - 0.5) * 2.,
            (0.5 - (y - top) / height) * 2.,
        )
            .into()
    }

    pub fn line_for_screen_point(&self, context: &Context, x: f32, y: f32) -> shared::math3::Line3 {
        let inverse = self.view_projection_matrix.inverse();
        let point = self.point_to_view_space(context, x, y);

        shared::math3::Line3::from((
            inverse * shared::math3::Vector3::from((point[0], point[1], 0.)),
            inverse * shared::math3::Vector3::from((point[0], point[1], 1.)),
        ))
    }

    pub fn trace_camera_plane(
        &self,
        context: &Context,
        x: f32,
        y: f32,
    ) -> Option<shared::math3::Vector3> {
        let line = self.line_for_screen_point(context, x, y);
        let plane =
            shared::math3::Plane3::new(shared::math3::Vector3::from((0., 0., 1.)), -self.target[2]);

        line.intersect(plane)
    }

    pub fn entities_in_rect(
        &self,
        context: &Context,
        rect: shared::math3::Rect,
        max: u32,
    ) -> Vec<u32> {
        if max == 0 {
            return Vec::new();
        }

        let a = self.point_to_view_space(context, rect.x, rect.y);
        let b = self.point_to_view_space(context, rect.x + rect.w, rect.y + rect.h);
        let screen = shared::math3::Rect::new(
            a[0].min(b[0]),
            a[1].min(b[1]),
            (a[0] - b[0]).abs(),
            (a[1] - b[1]).abs(),
        );

        self.entities
            .iter()
            .filter(|entity| {
                let origin = self.view_projection_matrix * entity.origin;
                entity.number != 0
                    && entity.flags
                        & (RenderEntityFlag::Hidden as u32 | RenderEntityFlag::NotSelectable as u32)
                        == 0
                    && screen.contains((origin[0], origin[1]).into())
            })
            .take(max as usize)
            .map(|entity| entity.number)
            .collect()
    }

    pub fn collect_revealers(&self, max_revealers: usize) -> Vec<&RenderEntity<'_>> {
        self.entities
            .iter()
            .filter(|entity| {
                entity.team == self.player as u32
                    && entity.flags & RenderEntityFlag::Hidden as u32 == 0
                    && entity.flags & RenderEntityFlag::FogOfWarRevealer as u32 != 0
            })
            .take(max_revealers)
            .collect()
    }
}
