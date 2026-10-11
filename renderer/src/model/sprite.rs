use crate::{
    context,
    model::mdx,
    render_entity::{RenderEntity, RenderEntityFlag},
    texture,
    view::{View, ViewCamera, ViewRDFlag},
};

pub struct Sprite<'s> {
    pub model: &'s mdx::Model,
    pub animation: &'s str,
    pub x: f32,
    pub y: f32,

    pub skin: Option<&'s texture::Texture>,
    pub skin_slot: u32,
    pub start_time: u32,
}

impl<'s> Sprite<'s> {
    pub fn draw(
        &self,
        context: &context::Context,
        time: u32,
        delta_time: u32,
        tint: Option<shared::color::Color32>,
    ) {
        let Some(seq) = self.model.select_ui_sequence(self.animation) else {
            return;
        };

        let fdf_sprite_coords = self.animation.starts_with("#!");
        let screen = context.ui_scene;

        let frame = seq.frame(self.animation, time - self.start_time);
        let entity = RenderEntity {
            origin: if fdf_sprite_coords {
                (self.x, self.y, 0.)
            } else {
                (self.x, screen.y + screen.h - self.y, 0.)
            }
            .into(),
            flags: RenderEntityFlag::NoFogOfWar as u32
                | RenderEntityFlag::NoShadow as u32
                | RenderEntityFlag::NoLighting as u32,
            skin: self.skin,
            skin_slot: self.skin_slot,
            frame,
            previous_frame: frame,
            tint: tint.unwrap_or(shared::color::Color32::white()),
            ..RenderEntity::new(self.model)
        };

        let view_projection_matrix = shared::math3::Matrix4::ortho(
            screen.x,
            screen.x + screen.w,
            screen.y,
            screen.y + screen.h,
            0.,
            100.,
        );
        let view_projection_matrix = view_projection_matrix.scale((1., 1., 0.).into());

        let view = View {
            scissor: shared::math3::Rect::new(0., 0., 1., 1.),
            entities: vec![entity],
            rdflags: ViewRDFlag::NoWorldModel as u32
                | ViewRDFlag::NoFrustumCull as u32
                | ViewRDFlag::UISprite as u32,
            viewport: shared::math3::Rect::new(0., 0., 1., 1.),
            view_projection_matrix,
            time,
            delta_time,
            camera_state: [
                ViewCamera {
                    eye: (0., 0., 100.).into(),
                    ..Default::default()
                },
                ViewCamera::default(),
            ],

            ..Default::default()
        };

        view.render(context);
    }
}
