use shared::BlendMode;

use crate::{particle::Particle, texture};

pub const PARTICLE_TEXTURES: u32 = 4;

#[derive(Default)]
pub struct Batch<'s> {
    pub textures: Vec<Option<&'s texture::Texture>>,

    pub first: u32,
    pub count: u32,
    pub texcount: u32,

    pub blend_mode: BlendMode,
}

impl<'s> Batch<'s> {
    pub fn clear(&mut self) {
        self.textures.clear();

        self.first = 0;
        self.count = 0;
        self.texcount = 0;

        self.blend_mode = BlendMode::None;
    }

    pub fn texture_slot(&mut self, particle: &'s Particle) -> u32 {
        let texture = particle.texture.as_ref();
        if self.count != 0 && self.blend_mode != particle.blend_mode {
            return PARTICLE_TEXTURES;
        }
        if let Some(index) = self
            .textures
            .iter()
            .position(|item| match (*item, texture) {
                (Some(item), Some(texture)) => item.is_same(texture),
                (None, None) => true,
                _ => false,
            })
        {
            return index as u32;
        }
        if self.texcount == PARTICLE_TEXTURES {
            return PARTICLE_TEXTURES;
        }

        self.blend_mode = particle.blend_mode;
        self.textures.push(texture);
        let slot = self.texcount;
        self.texcount += 1;
        slot
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reuses_default_texture_slot() {
        let particle = Particle::default();
        let mut batch = Batch {
            textures: Vec::new(),
            first: 0,
            count: 0,
            texcount: 0,
            blend_mode: BlendMode::Add,
        };

        assert_eq!(batch.texture_slot(&particle), 0);
        assert_eq!(batch.texture_slot(&particle), 0);
        assert_eq!(batch.texcount, 1);
        assert_eq!(batch.textures.len(), 1);
    }
}
