mod desc;
mod light;
mod program;

mod model_state;
mod particle_state;

mod particle_vertex;
mod trail_vertex;
mod vertex;

pub use desc::{
    Attribute, AttributeID, Descriptor, PrecisionType, ShaderDescriptor, Uniform, UniformType,
    Varying,
};
pub use program::Program;

pub use light::{EnvironLight, LightType, ModelLight, ModelLighting};

pub use model_state::ModelState;
pub use particle_state::ParticleState;
pub use particle_vertex::{ParticleQuad, ParticleUvOrder, ParticleVertex, ParticleVertices};

pub use trail_vertex::{TrailVertex, TrailVertices};
pub use vertex::{Vertex, Vertices};

pub mod builtin;
