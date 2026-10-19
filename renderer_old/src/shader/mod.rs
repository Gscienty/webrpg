mod desc;
mod error;
mod program;

mod grass;
mod light;

mod default_state;
mod fow_raycast_state;
mod model_state;
mod particle_state;
mod sprite_state;

mod builtin_default_shader_desc;
mod builtin_fow_raycast_shader_desc;
mod builtin_model_shader_desc;
mod builtin_particle_shader_desc;
mod builtin_sprite_shader_desc;

mod shader_type;

pub use desc::{Attribute, AttributeID, Descriptor, PrecisionType, Uniform, UniformType, Varying};
pub use error::Error;
pub use program::Program;

pub use grass::ModelGrass;
pub use light::{EnvironLight, LightType, ModelLight, ModelLighting};

pub use default_state::DefaultState;
pub use fow_raycast_state::FowRaycastState;
pub use model_state::ModelState;
pub use particle_state::ParticleState;
pub use sprite_state::SpriteState;

pub use builtin_default_shader_desc::DEFAULT_SHADER;
pub use builtin_fow_raycast_shader_desc::FOW_RAYCAST_SHADER;
pub use builtin_model_shader_desc::{MODEL_INSTANCED_SHADER, MODEL_SHADER};
pub use builtin_particle_shader_desc::PARTICLE_SHADER;
pub use builtin_sprite_shader_desc::{
    COMMANDBUTTON_SHADER, MINIMAP_FOG_SHADER, MINIMAP_SHADER, SHADOW_SPLAT_SHADER, SPLAT_SHADER,
    UNLIT_SHADER,
};

pub use shader_type::ShaderType;
