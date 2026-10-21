use shared::math3;

pub struct RenderParams<'s> {
    pub origin: math3::Vector3,
    pub angle: f32,
    pub scale: f32,

    pub no_frustum_cull: bool,

    pub lerpfrac: f32,

    pub global_time: u32,
    pub frame: u32,
    pub previous_frame: u32,

    pub view_projection: math3::Matrix4,
    pub frustum: &'s math3::Frustum3,
}

pub fn evaluate_sprite_transform(scale: f32, angle: f32, origin: math3::Vector3) -> math3::Matrix4 {
    let angle = math3::Orientation::from((angle, 0., 0.));
    let rotation = math3::Quaternion::from(&angle);

    math3::Matrix4::from_rotation_translation_scale_origin(
        rotation,
        origin,
        (scale, scale, scale).into(),
        (0., 0., 0.).into(),
    )
}
