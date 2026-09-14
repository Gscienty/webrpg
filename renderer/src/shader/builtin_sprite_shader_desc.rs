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

pub static MINIMAP: Descriptor<3, 3, 2> = Descriptor::new(
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
    "vec4 vert() {
    v_texcoord = a_texcoord;
    v_color = a_color;
    return u_view_projection * u_model * vec4(a_position, 1.0);
}",
    "vec4 frag() {
    float mask = 1.0 - smoothstep(0.49, 0.5, length(v_color.rg - vec2(0.5)));
    vec4 tex = texture(u_texture, v_texcoord);
    return vec4(tex.rgb, tex.a * mask);
}",
);

pub static SPLAT: Descriptor<3, 3, 2> = Descriptor::new(
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
    "vec4 vert() {
    v_texcoord = a_texcoord;
    v_color = a_color;
    return u_view_projection * u_model * vec4(a_position, 1.0);
}",
    "float crop_edges(vec2 tc) {
    return step(abs(tc.x - 0.5), 0.5) * step(abs(tc.y - 0.5), 0.5);
}
vec4 frag() {
    vec4 col = texture(u_texture, v_texcoord) * v_color;
    col.a *= crop_edges(v_texcoord);
    return col;
}",
);

pub static SHADOW_SPLAT: Descriptor<6, 3, 2> = Descriptor::new(
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
    "vec4 vert() {
    v_texcoord = a_texcoord;
    v_color = a_color;
    return u_view_projection * u_model * vec4(a_position, 1.0);
}",
    "float crop_edges(vec2 tc) {
    return step(abs(tc.x - 0.5), 0.5) * step(abs(tc.y - 0.5), 0.5);
}
vec4 frag() {
    vec4 tex = texture(u_texture, v_texcoord);
    vec4 col = vec4(0.0, 0.0, 0.0, tex.a * v_color.a * crop_edges(v_texcoord));
    if (u_fog_enable) {
        float fog_range = u_fog_params.y - u_fog_params.x;
        float depth = gl_FragCoord.z / gl_FragCoord.w;
        float fog_factor = abs(fog_range) > 0.0001
            ? clamp((u_fog_params.y - depth) / fog_range, 0.0, 1.0)
            : (depth <= u_fog_params.x ? 1.0 : 0.0);
        col.rgb = mix(u_fog_color, col.rgb, fog_factor);
    }
    return col;
}",
);

pub static COMMANDBUTTON: Descriptor<5, 3, 2> = Descriptor::new(
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
    "vec4 vert() {
    v_texcoord = a_texcoord;
    v_color = a_color;
    return u_view_projection * u_model * vec4(a_position, 1.0);
}",
    "vec4 frag() {
    vec4 col = texture(u_texture, v_texcoord) * v_color;
    float glow = max(abs(v_texcoord.x - 0.5), abs(v_texcoord.y - 0.5));
    glow = smoothstep(0.33, 0.5, glow) * 0.75 * u_active_glow;
    col.rgb = mix(col.rgb, vec3(0.5, 1.0, 0.5), glow);
    vec2 radial = v_texcoord - vec2(0.5);
    float angle = atan(radial.x, -radial.y) / 6.28318530718;
    angle = angle < 0.0 ? angle + 1.0 : angle;
    float elapsed = 1.0 - clamp(u_radial_shade, 0.0, 1.0);
    float shade = step(elapsed, angle) * step(0.000001, u_radial_shade);
    col.rgb *= mix(1.0, 0.35, shade);
    float crop = step(abs(v_texcoord.x - 0.5), 0.5) * step(abs(v_texcoord.y - 0.5), 0.5);
    col.a *= crop;
    return col;
}",
);

pub static MINIMAP_FOG: Descriptor<3, 3, 2> = Descriptor::new(
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
    "vec4 vert() {
    v_texcoord = a_texcoord;
    v_color = a_color;
    return u_view_projection * u_model * vec4(a_position, 1.0);
}",
    "vec4 frag() {
    float visibility = texture(u_texture, vec2(v_texcoord.x, 1.0 - v_texcoord.y)).r;
    float alpha = clamp(1.0 - visibility, 0.0, 1.0) * v_color.a;
    return vec4(v_color.rgb, alpha);
}",
);
