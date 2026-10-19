use crate::{
    attribute_declare,
    shader::{AttributeID, Descriptor, PrecisionType, SpriteState, UniformType},
    uniform_declare, varying_declare,
};

pub static UNLIT_SHADER: Descriptor<3, 3, 2> = Descriptor::new(
    "unlit",
    [
        uniform_declare!(
            SpriteState,
            view_projection,
            UniformType::FloatMat4,
            PrecisionType::High
        ),
        uniform_declare!(
            SpriteState,
            model,
            UniformType::FloatMat4,
            PrecisionType::High
        ),
        uniform_declare!(
            SpriteState,
            texture,
            UniformType::Sampler2D,
            PrecisionType::Low
        ),
    ],
    [
        attribute_declare!(position, AttributeID::Position, UniformType::FloatVec3),
        attribute_declare!(texcoord, AttributeID::Texcoord, UniformType::FloatVec2),
        attribute_declare!(color, AttributeID::Color, UniformType::Color),
    ],
    [
        varying_declare!(texcoord, UniformType::FloatVec2),
        varying_declare!(color, UniformType::Color),
    ],
    include_str!("glsl/unlit.vert.glsl"),
    include_str!("glsl/unlit.frag.glsl"),
);

pub static MINIMAP_SHADER: Descriptor<3, 3, 2> = Descriptor::new(
    "minimap",
    [
        uniform_declare!(
            SpriteState,
            view_projection,
            UniformType::FloatMat4,
            PrecisionType::High
        ),
        uniform_declare!(
            SpriteState,
            model,
            UniformType::FloatMat4,
            PrecisionType::High
        ),
        uniform_declare!(
            SpriteState,
            texture,
            UniformType::Sampler2D,
            PrecisionType::Low
        ),
    ],
    [
        attribute_declare!(position, AttributeID::Position, UniformType::FloatVec3),
        attribute_declare!(texcoord, AttributeID::Texcoord, UniformType::FloatVec2),
        attribute_declare!(color, AttributeID::Color, UniformType::Color),
    ],
    [
        varying_declare!(texcoord, UniformType::FloatVec2),
        varying_declare!(color, UniformType::Color),
    ],
    include_str!("glsl/minimap.vert.glsl"),
    include_str!("glsl/minimap.frag.glsl"),
);

pub static SPLAT_SHADER: Descriptor<3, 3, 2> = Descriptor::new(
    "splat",
    [
        uniform_declare!(
            SpriteState,
            view_projection,
            UniformType::FloatMat4,
            PrecisionType::High
        ),
        uniform_declare!(
            SpriteState,
            model,
            UniformType::FloatMat4,
            PrecisionType::High
        ),
        uniform_declare!(
            SpriteState,
            texture,
            UniformType::Sampler2D,
            PrecisionType::Low
        ),
    ],
    [
        attribute_declare!(position, AttributeID::Position, UniformType::FloatVec3),
        attribute_declare!(texcoord, AttributeID::Texcoord, UniformType::FloatVec2),
        attribute_declare!(color, AttributeID::Color, UniformType::Color),
    ],
    [
        varying_declare!(texcoord, UniformType::FloatVec2),
        varying_declare!(color, UniformType::Color),
    ],
    include_str!("glsl/splat.vert.glsl"),
    include_str!("glsl/splat.frag.glsl"),
);

pub static SHADOW_SPLAT_SHADER: Descriptor<6, 3, 2> = Descriptor::new(
    "shadow_splat",
    [
        uniform_declare!(
            SpriteState,
            view_projection,
            UniformType::FloatMat4,
            PrecisionType::High
        ),
        uniform_declare!(
            SpriteState,
            model,
            UniformType::FloatMat4,
            PrecisionType::High
        ),
        uniform_declare!(
            SpriteState,
            texture,
            UniformType::Sampler2D,
            PrecisionType::Low
        ),
        uniform_declare!(
            SpriteState,
            fog_enable,
            UniformType::Bool,
            PrecisionType::Low
        ),
        uniform_declare!(
            SpriteState,
            fog_color,
            UniformType::FloatVec3,
            PrecisionType::Low
        ),
        uniform_declare!(
            SpriteState,
            fog_params,
            UniformType::FloatVec2,
            PrecisionType::High
        ),
    ],
    [
        attribute_declare!(position, AttributeID::Position, UniformType::FloatVec3),
        attribute_declare!(texcoord, AttributeID::Texcoord, UniformType::FloatVec2),
        attribute_declare!(color, AttributeID::Color, UniformType::Color),
    ],
    [
        varying_declare!(texcoord, UniformType::FloatVec2),
        varying_declare!(color, UniformType::Color),
    ],
    include_str!("glsl/shadow_splat.vert.glsl"),
    include_str!("glsl/shadow_splat.frag.glsl"),
);

pub static COMMANDBUTTON_SHADER: Descriptor<5, 3, 2> = Descriptor::new(
    "commandbutton",
    [
        uniform_declare!(
            SpriteState,
            view_projection,
            UniformType::FloatMat4,
            PrecisionType::High
        ),
        uniform_declare!(
            SpriteState,
            model,
            UniformType::FloatMat4,
            PrecisionType::High
        ),
        uniform_declare!(
            SpriteState,
            texture,
            UniformType::Sampler2D,
            PrecisionType::Low
        ),
        uniform_declare!(
            SpriteState,
            active_glow,
            UniformType::Float,
            PrecisionType::Low
        ),
        uniform_declare!(
            SpriteState,
            radial_shade,
            UniformType::Float,
            PrecisionType::Low
        ),
    ],
    [
        attribute_declare!(position, AttributeID::Position, UniformType::FloatVec3),
        attribute_declare!(texcoord, AttributeID::Texcoord, UniformType::FloatVec2),
        attribute_declare!(color, AttributeID::Color, UniformType::Color),
    ],
    [
        varying_declare!(texcoord, UniformType::FloatVec2),
        varying_declare!(color, UniformType::Color),
    ],
    include_str!("glsl/commandbutton.vert.glsl"),
    include_str!("glsl/commandbutton.frag.glsl"),
);

pub static MINIMAP_FOG_SHADER: Descriptor<3, 3, 2> = Descriptor::new(
    "minimap_fog",
    [
        uniform_declare!(
            SpriteState,
            view_projection,
            UniformType::FloatMat4,
            PrecisionType::High
        ),
        uniform_declare!(
            SpriteState,
            model,
            UniformType::FloatMat4,
            PrecisionType::High
        ),
        uniform_declare!(
            SpriteState,
            texture,
            UniformType::Sampler2D,
            PrecisionType::Low
        ),
    ],
    [
        attribute_declare!(position, AttributeID::Position, UniformType::FloatVec3),
        attribute_declare!(texcoord, AttributeID::Texcoord, UniformType::FloatVec2),
        attribute_declare!(color, AttributeID::Color, UniformType::Color),
    ],
    [
        varying_declare!(texcoord, UniformType::FloatVec2),
        varying_declare!(color, UniformType::Color),
    ],
    include_str!("glsl/minimap_fog.vert.glsl"),
    include_str!("glsl/minimap_fog.frag.glsl"),
);
