use crate::{
    attribute_declare,
    shader::{AttributeID, Descriptor, FowRaycastState, PrecisionType, UniformType},
    uniform_declare, varying_declare,
};

pub static FOW_RAYCAST_SHADER: Descriptor<3, 3, 2> = Descriptor::new(
    "fow_raycast",
    [
        uniform_declare!(
            FowRaycastState,
            view_projection,
            UniformType::FloatMat4,
            PrecisionType::High
        ),
        uniform_declare!(
            FowRaycastState,
            model,
            UniformType::FloatMat4,
            PrecisionType::High
        ),
        uniform_declare!(
            FowRaycastState,
            eye_position,
            UniformType::FloatVec2,
            PrecisionType::Low
        ),
    ],
    [
        attribute_declare!(position, AttributeID::Position, UniformType::FloatVec3),
        attribute_declare!(color, AttributeID::Color, UniformType::Color),
        attribute_declare!(texcoord, AttributeID::Texcoord, UniformType::FloatVec2),
    ],
    [
        varying_declare!(color, UniformType::Color),
        varying_declare!(texcoord, UniformType::FloatVec2),
    ],
    include_str!("glsl/fow_raycast.vert.glsl"),
    include_str!("glsl/fow_raycast.frag.glsl"),
);
