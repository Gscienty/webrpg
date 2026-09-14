use crate::{
    attribute_declare,
    shader::{AttributeID, Descriptor, ModelState, PrecisionType, UniformType},
    uniform_declare, varying_declare,
};

pub static MODEL: Descriptor<21, 6, 6> = Descriptor::new(
    "model",
    [
        uniform_declare!(
            ModelState,
            bones,
            UniformType::FloatMat4,
            PrecisionType::High,
            128,
            bone_count
        ),
        uniform_declare!(
            ModelState,
            view_projection,
            UniformType::FloatMat4,
            PrecisionType::High
        ),
        uniform_declare!(
            ModelState,
            light_matrix,
            UniformType::FloatMat4,
            PrecisionType::High
        ),
        uniform_declare!(
            ModelState,
            texture_matrix,
            UniformType::FloatMat4,
            PrecisionType::High
        ),
        uniform_declare!(
            ModelState,
            light_count,
            UniformType::Int,
            PrecisionType::Low
        ),
        uniform_declare!(
            ModelState,
            first_bone_lookup_index,
            UniformType::Float,
            PrecisionType::Low
        ),
        uniform_declare!(
            ModelState,
            lights,
            UniformType::FloatMat4,
            PrecisionType::High,
            8
        ),
        uniform_declare!(
            ModelState,
            model,
            UniformType::FloatMat4,
            PrecisionType::High
        ),
        uniform_declare!(
            ModelState,
            normal_matrix,
            UniformType::FloatMat3Transpose,
            PrecisionType::High
        ),
        uniform_declare!(
            ModelState,
            texture,
            UniformType::Sampler2D,
            PrecisionType::Low
        ),
        uniform_declare!(
            ModelState,
            shadowmap,
            UniformType::Sampler2D,
            PrecisionType::Low
        ),
        uniform_declare!(
            ModelState,
            fog_of_war,
            UniformType::Sampler2D,
            PrecisionType::Low
        ),
        uniform_declare!(
            ModelState,
            layer_alpha,
            UniformType::Float,
            PrecisionType::Low
        ),
        uniform_declare!(
            ModelState,
            geoset_color,
            UniformType::FloatVec4,
            PrecisionType::Low
        ),
        uniform_declare!(
            ModelState,
            uv_matrix,
            UniformType::FloatMat3,
            PrecisionType::High
        ),
        uniform_declare!(ModelState, alpha_key, UniformType::Bool, PrecisionType::Low),
        uniform_declare!(
            ModelState,
            alpha_cutoff,
            UniformType::Float,
            PrecisionType::Low
        ),
        uniform_declare!(ModelState, unshaded, UniformType::Bool, PrecisionType::Low),
        uniform_declare!(
            ModelState,
            fog_enable,
            UniformType::Bool,
            PrecisionType::Low
        ),
        uniform_declare!(
            ModelState,
            fog_color,
            UniformType::FloatVec3,
            PrecisionType::Low
        ),
        uniform_declare!(
            ModelState,
            fog_params,
            UniformType::FloatVec2,
            PrecisionType::Low
        ),
    ],
    [
        attribute_declare!(position, AttributeID::Position, UniformType::FloatVec3),
        attribute_declare!(color, AttributeID::Color, UniformType::Color),
        attribute_declare!(texcoord, AttributeID::Texcoord, UniformType::FloatVec2),
        attribute_declare!(normal, AttributeID::Normal, UniformType::FloatVec3),
        attribute_declare!(skin1, AttributeID::Skin, UniformType::FloatVec4),
        attribute_declare!(
            bone_weight1,
            AttributeID::BoneWeight,
            UniformType::FloatVec4
        ),
    ],
    [
        varying_declare!(color, UniformType::Color),
        varying_declare!(shadow, UniformType::FloatVec4),
        varying_declare!(shadowlight, UniformType::FloatVec3),
        varying_declare!(texcoord, UniformType::FloatVec2),
        varying_declare!(texcoord2, UniformType::FloatVec2),
        varying_declare!(lighting, UniformType::FloatVec3),
    ],
    include_str!("glsl/model.vert.glsl"),
    include_str!("glsl/model.frag.glsl"),
);

pub static MODEL_INSTANCED: Descriptor<20, 7, 6> = Descriptor::new(
    "model_instanced",
    [
        uniform_declare!(
            ModelState,
            bones,
            UniformType::FloatMat4,
            PrecisionType::High,
            128,
            bone_count
        ),
        uniform_declare!(
            ModelState,
            view_projection,
            UniformType::FloatMat4,
            PrecisionType::High
        ),
        uniform_declare!(
            ModelState,
            light_matrix,
            UniformType::FloatMat4,
            PrecisionType::High
        ),
        uniform_declare!(
            ModelState,
            texture_matrix,
            UniformType::FloatMat4,
            PrecisionType::High
        ),
        uniform_declare!(
            ModelState,
            light_count,
            UniformType::Int,
            PrecisionType::Low
        ),
        uniform_declare!(
            ModelState,
            first_bone_lookup_index,
            UniformType::Float,
            PrecisionType::Low
        ),
        uniform_declare!(
            ModelState,
            lights,
            UniformType::FloatMat4,
            PrecisionType::High,
            8
        ),
        uniform_declare!(
            ModelState,
            grass_params,
            UniformType::FloatMat4,
            PrecisionType::High
        ),
        uniform_declare!(
            ModelState,
            texture,
            UniformType::Sampler2D,
            PrecisionType::Low
        ),
        uniform_declare!(
            ModelState,
            shadowmap,
            UniformType::Sampler2D,
            PrecisionType::Low
        ),
        uniform_declare!(
            ModelState,
            fog_of_war,
            UniformType::Sampler2D,
            PrecisionType::Low
        ),
        uniform_declare!(
            ModelState,
            layer_alpha,
            UniformType::Float,
            PrecisionType::Low
        ),
        uniform_declare!(
            ModelState,
            geoset_color,
            UniformType::FloatVec4,
            PrecisionType::Low
        ),
        uniform_declare!(
            ModelState,
            uv_matrix,
            UniformType::FloatMat3,
            PrecisionType::High
        ),
        uniform_declare!(ModelState, alpha_key, UniformType::Bool, PrecisionType::Low),
        uniform_declare!(
            ModelState,
            alpha_cutoff,
            UniformType::Float,
            PrecisionType::Low
        ),
        uniform_declare!(ModelState, unshaded, UniformType::Bool, PrecisionType::Low),
        uniform_declare!(
            ModelState,
            fog_enable,
            UniformType::Bool,
            PrecisionType::Low
        ),
        uniform_declare!(
            ModelState,
            fog_color,
            UniformType::FloatVec3,
            PrecisionType::Low
        ),
        uniform_declare!(
            ModelState,
            fog_params,
            UniformType::FloatVec2,
            PrecisionType::Low
        ),
    ],
    [
        attribute_declare!(position, AttributeID::Position, UniformType::FloatVec3),
        attribute_declare!(color, AttributeID::Color, UniformType::Color),
        attribute_declare!(texcoord, AttributeID::Texcoord, UniformType::FloatVec2),
        attribute_declare!(normal, AttributeID::Normal, UniformType::FloatVec3),
        attribute_declare!(skin1, AttributeID::Skin, UniformType::FloatVec4),
        attribute_declare!(
            bone_weight1,
            AttributeID::BoneWeight,
            UniformType::FloatVec4
        ),
        attribute_declare!(instance, AttributeID::Instance, UniformType::FloatMat4),
    ],
    [
        varying_declare!(color, UniformType::Color),
        varying_declare!(shadow, UniformType::FloatVec4),
        varying_declare!(shadowlight, UniformType::FloatVec3),
        varying_declare!(texcoord, UniformType::FloatVec2),
        varying_declare!(texcoord2, UniformType::FloatVec2),
        varying_declare!(lighting, UniformType::FloatVec3),
    ],
    include_str!("glsl/model_instanced.vert.glsl"),
    include_str!("glsl/model.frag.glsl"),
);
