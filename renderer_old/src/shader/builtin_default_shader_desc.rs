use crate::{
    attribute_declare,
    shader::{AttributeID, DefaultState, Descriptor, PrecisionType, UniformType},
    uniform_declare, varying_declare,
};

pub static DEFAULT_SHADER: Descriptor<13, 4, 8> = Descriptor::new(
    "default",
    [
        uniform_declare!(
            DefaultState,
            view_projection,
            UniformType::FloatMat4,
            PrecisionType::High
        ),
        uniform_declare!(
            DefaultState,
            texture_matrix,
            UniformType::FloatMat4,
            PrecisionType::High
        ),
        uniform_declare!(
            DefaultState,
            model,
            UniformType::FloatMat4,
            PrecisionType::High
        ),
        uniform_declare!(
            DefaultState,
            light_matrix,
            UniformType::FloatMat4,
            PrecisionType::High
        ),
        uniform_declare!(
            DefaultState,
            normal_matrix,
            UniformType::FloatMat3Transpose,
            PrecisionType::High
        ),
        uniform_declare!(
            DefaultState,
            light_count,
            UniformType::Int,
            PrecisionType::Low
        ),
        uniform_declare!(
            DefaultState,
            lights,
            UniformType::FloatMat4,
            PrecisionType::High,
            8
        ),
        uniform_declare!(
            DefaultState,
            texture,
            UniformType::Sampler2D,
            PrecisionType::Low
        ),
        uniform_declare!(
            DefaultState,
            shadowmap,
            UniformType::Sampler2D,
            PrecisionType::Low
        ),
        uniform_declare!(
            DefaultState,
            fog_of_war,
            UniformType::Sampler2D,
            PrecisionType::Low
        ),
        uniform_declare!(
            DefaultState,
            fog_enable,
            UniformType::Bool,
            PrecisionType::Low
        ),
        uniform_declare!(
            DefaultState,
            fog_color,
            UniformType::FloatVec3,
            PrecisionType::Low
        ),
        uniform_declare!(
            DefaultState,
            fog_params,
            UniformType::FloatVec2,
            PrecisionType::Low
        ),
    ],
    [
        attribute_declare!(position, AttributeID::Position, UniformType::FloatVec3),
        attribute_declare!(texcoord, AttributeID::Texcoord, UniformType::FloatVec2),
        attribute_declare!(normal, AttributeID::Normal, UniformType::FloatVec3),
        attribute_declare!(color, AttributeID::Color, UniformType::Color),
    ],
    [
        varying_declare!(texcoord, UniformType::FloatVec2),
        varying_declare!(texcoord2, UniformType::FloatVec2),
        varying_declare!(normal, UniformType::FloatVec3),
        varying_declare!(light_dir, UniformType::FloatVec3),
        varying_declare!(lighting, UniformType::FloatVec3),
        varying_declare!(shadowlight, UniformType::FloatVec3),
        varying_declare!(color, UniformType::Color),
        varying_declare!(shadow, UniformType::FloatVec4),
    ],
    include_str!("glsl/default.vert.glsl"),
    include_str!("glsl/default.frag.glsl"),
);
