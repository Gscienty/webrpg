use crate::{
    attribute_declare,
    shader::{AttributeID, Descriptor, ParticleState, PrecisionType, UniformType},
    uniform_declare, varying_declare,
};

pub static PARTICLE_SHADER: Descriptor<9, 8, 3> = Descriptor::new(
    "particle",
    [
        uniform_declare!(
            ParticleState,
            view_projection,
            UniformType::FloatMat4,
            PrecisionType::High
        ),
        uniform_declare!(
            ParticleState,
            texture_matrix,
            UniformType::FloatMat4,
            PrecisionType::High
        ),
        uniform_declare!(
            ParticleState,
            model,
            UniformType::FloatMat4,
            PrecisionType::High
        ),
        uniform_declare!(
            ParticleState,
            eye,
            UniformType::FloatVec3,
            PrecisionType::High
        ),
        uniform_declare!(
            ParticleState,
            texture,
            UniformType::Sampler2D,
            PrecisionType::Low,
            4
        ),
        uniform_declare!(
            ParticleState,
            fog_of_war,
            UniformType::Sampler2D,
            PrecisionType::Low
        ),
        uniform_declare!(
            ParticleState,
            alpha_key,
            UniformType::Bool,
            PrecisionType::Low
        ),
        uniform_declare!(
            ParticleState,
            alpha_cutoff,
            UniformType::Float,
            PrecisionType::Low
        ),
        uniform_declare!(
            ParticleState,
            modulate,
            UniformType::Int,
            PrecisionType::Low
        ),
    ],
    [
        attribute_declare!(position, AttributeID::Position, UniformType::FloatVec3),
        attribute_declare!(color, AttributeID::Color, UniformType::Color),
        attribute_declare!(texcoord, AttributeID::Texcoord, UniformType::FloatVec3),
        attribute_declare!(size, AttributeID::ParticleSize, UniformType::Float),
        attribute_declare!(tail, AttributeID::ParticleTail, UniformType::FloatVec3),
        attribute_declare!(
            quad_right,
            AttributeID::ParticleQuadRight,
            UniformType::FloatVec3
        ),
        attribute_declare!(quad_up, AttributeID::ParticleQuadUp, UniformType::FloatVec3),
        attribute_declare!(axis, AttributeID::ParticleAxis, UniformType::FloatVec2),
    ],
    [
        varying_declare!(color, UniformType::Color),
        varying_declare!(texcoord, UniformType::FloatVec3),
        varying_declare!(texcoord2, UniformType::FloatVec2),
    ],
    include_str!("../glsl/particle.vert.glsl"),
    include_str!("../glsl/particle.frag.glsl"),
);
