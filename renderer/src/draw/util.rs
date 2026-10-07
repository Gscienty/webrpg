pub const DRAW_CLIP: u8 = 1 << 0;

pub fn scene_rect_for_size(drawable_width: u32, drawable_height: u32) -> shared::math3::Rect {
    if drawable_height > 0 {
        let aspect = drawable_width as f32 / drawable_height as f32;

        if aspect > 4. / 3. {
            return shared::math3::Rect::new(0., 0., 0.8 * aspect, 0.6 * aspect);
        }
    }

    shared::math3::Rect::new(0., 0., 0.8, 0.6)
}

pub fn ui_clip_scissor(
    drawable_width: u32,
    drawable_height: u32,
    clip: &shared::math3::Rect,
) -> (i32, i32, i32, i32) {
    let scene = scene_rect_for_size(drawable_width, drawable_height);
    let x = ((clip.x - scene.x) / scene.w).clamp(0., 1.);
    let y = (1. - (clip.y + clip.h - scene.y) / scene.h).clamp(0., 1.);
    let w = (clip.w / scene.w).clamp(0., 1. - x);
    let h = (clip.h / scene.h).clamp(0., 1. - y);

    (
        (x * drawable_width as f32) as i32,
        (y * drawable_height as f32) as i32,
        (w * drawable_width as f32) as i32,
        (h * drawable_height as f32) as i32,
    )
}
