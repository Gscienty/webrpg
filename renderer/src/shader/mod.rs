mod desc;
mod light;
mod model_state;
mod program;
mod vertex;

pub use desc::{
    Attribute, AttributeID, Descriptor, PrecisionType, ShaderDescriptor, Uniform, UniformType,
    Varying,
};
pub use program::Program;
pub use vertex::{Vertex, Vertices};

pub use light::{EnvironLight, LightType, ModelLight, ModelLighting};

pub use model_state::ModelState;

pub mod builtin;
