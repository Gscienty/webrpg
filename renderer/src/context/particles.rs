use crate::particles::Particle;

pub struct Particles {
    pub active: Vec<Vec<Particle>>,
}

impl Particles {
    pub fn init() -> Self {
        Self {
            active: vec![Vec::new()],
        }
    }
}
