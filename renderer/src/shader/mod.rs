mod builtin_shader_desc;
mod desc;
mod sprite_state;

pub use desc::{Attribute, AttributeID, Descriptor, PrecisionType, Uniform, UniformType, Varying};
pub use sprite_state::SpriteState;

pub use builtin_shader_desc::UNLIT;
