use crate::{
    attribute_declare,
    shader::{AttributeID, Descriptor, PrecisionType, SpriteState, UniformType},
    uniform_declare, varying_declare,
};

pub static UNLIT: Descriptor<3, 3, 2> = Descriptor::new(
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
    "vec4 vert() {
    v_texcoord = a_texcoord;
    v_color = a_color;
    return u_view_projection * u_model * vec4(a_position, 1.0);
}",
    "vec4 frag() {
    return texture(u_texture, v_texcoord) * v_color;
}",
);
