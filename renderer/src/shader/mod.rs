mod desc;

mod default_state;
mod model_state;
mod sprite_state;

mod builtin_default_shader_desc;
mod builtin_model_shader_desc;
mod builtin_sprite_shader_desc;

pub use desc::{Attribute, AttributeID, Descriptor, PrecisionType, Uniform, UniformType, Varying};

pub use default_state::DefaultState;
pub use model_state::ModelState;
pub use sprite_state::SpriteState;

pub use builtin_default_shader_desc::DEFAULT;
pub use builtin_model_shader_desc::{MODEL, MODEL_INSTANCED};
pub use builtin_sprite_shader_desc::{
    COMMANDBUTTON, MINIMAP, MINIMAP_FOG, SHADOW_SPLAT, SPLAT, UNLIT,
};
