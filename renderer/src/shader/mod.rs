mod desc;
mod program;
mod vertex;

pub use desc::{
    Attribute, AttributeID, Descriptor, PrecisionType, ShaderDescriptor, Uniform, UniformType,
    Varying,
};
pub use program::Program;
pub use vertex::{Vertex, Vertices};
