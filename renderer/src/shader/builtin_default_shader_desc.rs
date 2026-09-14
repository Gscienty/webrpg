use crate::{
    attribute_declare,
    shader::{AttributeID, DefaultState, Descriptor, PrecisionType, UniformType},
    uniform_declare, varying_declare,
};

pub static DEFAULT: Descriptor<13, 4, 8> = Descriptor::new(
    "default",
    [
        uniform_declare!(DefaultState, view_projection, UniformType::FloatMat4, PrecisionType::High),
        uniform_declare!(DefaultState, texture_matrix, UniformType::FloatMat4, PrecisionType::High),
        uniform_declare!(DefaultState, model, UniformType::FloatMat4, PrecisionType::High),
        uniform_declare!(DefaultState, light_matrix, UniformType::FloatMat4, PrecisionType::High),
        uniform_declare!(DefaultState, normal_matrix, UniformType::FloatMat3Transpose, PrecisionType::High),
        uniform_declare!(DefaultState, light_count, UniformType::Int, PrecisionType::Low),
        uniform_declare!(DefaultState, lights, UniformType::FloatMat4, PrecisionType::High, 8),
        uniform_declare!(DefaultState, texture, UniformType::Sampler2D, PrecisionType::Low),
        uniform_declare!(DefaultState, shadowmap, UniformType::Sampler2D, PrecisionType::Low),
        uniform_declare!(DefaultState, fog_of_war, UniformType::Sampler2D, PrecisionType::Low),
        uniform_declare!(DefaultState, fog_enable, UniformType::Bool, PrecisionType::Low),
        uniform_declare!(DefaultState, fog_color, UniformType::FloatVec3, PrecisionType::Low),
        uniform_declare!(DefaultState, fog_params, UniformType::FloatVec2, PrecisionType::Low),
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
    "const int MODEL_LIGHT_OMNI = 0;
const int MODEL_LIGHT_DIRECT = 1;
const int MODEL_LIGHT_AMBIENT = 2;
vec3 apply_environment_light(mat4 light, vec3 n, vec3 world_pos) {
    int type = int(light[0].w + 0.5);
    vec3 color = light[2].rgb * light[2].a;
    vec3 ambient = light[3].rgb * light[3].a;
    if (type == MODEL_LIGHT_AMBIENT) return color + ambient;
    if (type == MODEL_LIGHT_DIRECT) {
        vec3 l = normalize(-light[1].xyz);
        return clamp(color * max(dot(n, l), 0.0), vec3(0.0), vec3(1.0)) + ambient;
    }
    vec3 delta = light[0].xyz - world_pos;
    vec3 l = normalize(delta);
    float dist = length(delta) / 64.0 + 1.0;
    float atten = 1.0 / (dist * dist);
    return clamp(color * atten * max(dot(n, l), 0.0), vec3(0.0), vec3(1.0)) + ambient * atten;
}
vec3 environment_lighting(vec3 normal, vec3 world_pos) {
    vec3 n = normalize(normal);
    vec3 result = vec3(0.0);
    v_shadowlight = vec3(0.0);
    for (int i = 0; i < 8; ++i) {
        if (i >= u_light_count) break;
        vec3 contribution = apply_environment_light(u_lights[i], n, world_pos);
        result += contribution;
        if (i == 0 && int(u_lights[i][0].w + 0.5) == MODEL_LIGHT_DIRECT)
            v_shadowlight = contribution - u_lights[i][3].rgb * u_lights[i][3].a;
    }
    return result;
}
vec4 vert() {
    vec4 pos = u_model * vec4(a_position, 1.0);
    v_texcoord = a_texcoord;
    v_texcoord2 = (u_texture_matrix * pos).xy;
    v_normal = normalize(u_normal_matrix * a_normal);
    v_shadowlight = vec3(0.0);
    v_lighting = u_light_count > 0 ? environment_lighting(v_normal, pos.xyz) : vec3(0.0);
    v_shadow = u_light_matrix * pos;
    v_color = a_color;
    v_light_dir = -normalize(vec3(u_light_matrix[0][2], u_light_matrix[1][2], u_light_matrix[2][2])) * 1.2;
    return u_view_projection * pos;
}",
    "float get_light() {
    return dot(v_normal, v_light_dir);
}
float get_shadow() {
    float depth = texture(u_shadowmap, vec2(v_shadow.x + 1.0, v_shadow.y + 1.0) * 0.5).r;
    return depth < (v_shadow.z + 0.99) * 0.5 ? 0.0 : 1.0;
}
vec3 get_lighting() {
    if (u_light_count > 0) return clamp(v_lighting - v_shadowlight * (1.0 - get_shadow()), vec3(0.0), vec3(1.0));
    return vec3(min(1.0, mix(0.35, 1.0, get_shadow() * get_light()) * 1.1));
}
float get_fogofwar() {
    return texture(u_fog_of_war, v_texcoord2).r;
}
vec4 frag() {
    vec4 col = texture(u_texture, v_texcoord) * v_color;
    col.rgb *= get_fogofwar() * get_lighting();
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
