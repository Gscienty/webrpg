#[derive(Clone, Copy, PartialEq, Eq)]
pub enum RenderPhase {
    Solid,
    Lights,
    Alpha,
}

impl RenderPhase {
    pub const fn includes_pass(&self, blended_pass: bool, translucent_instance: bool) -> bool {
        if matches!(self, Self::Lights) {
            return true;
        }
        if matches!(self, Self::Alpha) {
            return blended_pass || translucent_instance;
        }

        !blended_pass && !translucent_instance
    }
}
