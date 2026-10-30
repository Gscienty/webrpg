use std::collections::BTreeMap;

use fileformat::mdx;
use shared::math3;

use crate::sprite::{RenderParams, RenderState, Sprite};

impl<'s> Sprite<'s> {
    pub(super) fn render_particle(
        &self,
        render_param: &RenderParams,
        render_state: &mut RenderState,
        transform: math3::Matrix4,
        node_matrices: &BTreeMap<u32, math3::Matrix4>,
    ) {
        let model: &'s mdx::Model<'s> = unsafe { &*(self.model.as_ref().get_ref() as *const _) };
        if render_param.emission_paused
            || (render_param.not_selectable && render_param.previous_frame == render_param.frame)
            || one_shot_emitter_finished(model, render_param.previous_frame, render_param.frame)
        {
            return;
        }

        let frame = (render_param.previous_frame as f32
            + (render_param.frame as f32 - render_param.previous_frame as f32)
                * render_param.lerpfrac) as u32;
        for emitter in &model.emitters2 {
            let visibility = emitter
                .visibility
                .as_ref()
                .and_then(|track| track.get_value(render_param.global_time, render_param.frame))
                .unwrap_or(1.);
            if visibility < f32::EPSILON {
                continue;
            }

            render_state.particle.emit(
                model,
                emitter,
                &self.textures,
                render_param.texture_set,
                transform,
                node_matrices,
                render_param.team,
                render_param.global_time,
                frame,
                render_param.delta,
            );
        }
    }
}

fn one_shot_emitter_finished(model: &mdx::Model, previous_frame: u32, frame: u32) -> bool {
    if previous_frame != frame {
        return false;
    }

    model.sequences.iter().any(|sequence| {
        sequence.interval[0] <= frame
            && frame < sequence.interval[1]
            && sequence.flags & 1 != 0
            && sequence.interval[1] > sequence.interval[0]
            && frame == sequence.interval[1] - 1
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_sequence_is_not_a_finished_one_shot() {
        assert!(!one_shot_emitter_finished(&mdx::Model::default(), 10, 10));
    }
}
