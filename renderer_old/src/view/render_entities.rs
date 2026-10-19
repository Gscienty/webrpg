use shared::math3::FrustumContains;

use crate::{
    context::{Context, RenderPhase},
    render_entity::{RenderEntity, RenderEntityFlag},
    view::{View, ViewRDFlag},
};

impl View<'_> {
    pub(super) fn render_entities(&self, context: &Context) {
        // TODO draw entity shadows

        for entity in self.entities.iter() {
            // TODO event

            if !self.entity_in_view(entity) {
                continue;
            }

            self.render_entity(context, entity);
        }
    }

    fn render_entity(&self, context: &Context, entity: &RenderEntity<'_>) {
        if entity.flags & RenderEntityFlag::Hidden as u32 != 0 {
            return;
        }

        if context.render_phase == RenderPhase::Lights {
            if entity.flags & RenderEntityFlag::NoShadow as u32 != 0 {
                entity.render(self, context);
            }
            return;
        }

        entity.render(self, context);
    }

    fn entity_in_view(&self, entity: &RenderEntity<'_>) -> bool {
        if entity.flags & RenderEntityFlag::Hidden as u32 != 0 {
            return false;
        }

        if self.rdflags & ViewRDFlag::NoFrustumCull as u32 != 0 {
            return true;
        }

        let bound = entity.model.bounds.box_;
        let matrix = entity.get_entity_matrix();

        self.frustum.contains((bound, matrix))
    }
}
