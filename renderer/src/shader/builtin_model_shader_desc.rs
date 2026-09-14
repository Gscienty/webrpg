use crate::{
    attribute_declare,
    shader::{AttributeID, Descriptor, ModelState, PrecisionType, UniformType},
    uniform_declare, varying_declare,
};

const MODEL_INSTANCED_VERTEX_BODY: &str = "const int MODEL_LIGHT_DIRECT = 1;
vec3 apply_light(mat4 light, vec3 n, vec3 world_pos) {
    int type = int(light[0].w + 0.5);
    vec3 color = light[2].rgb * light[2].a;
    vec3 ambient = light[3].rgb * light[3].a;
    if (type == 2) return color + ambient;
    vec3 delta = type == MODEL_LIGHT_DIRECT ? -light[1].xyz : light[0].xyz - world_pos;
    vec3 l = normalize(delta);
    float atten = type == MODEL_LIGHT_DIRECT ? 1.0 : 1.0 / pow(length(delta) / 64.0 + 1.0, 2.0);
    return clamp(color * atten * max(dot(n, l), 0.0), vec3(0.0), vec3(1.0)) + ambient * atten;
}
vec3 vertex_lighting(vec3 normal, vec3 world_pos) {
    vec3 result = vec3(0.0);
    v_shadowlight = vec3(0.0);
    for (int i = 0; i < 8; ++i) {
        if (i >= u_light_count) break;
        vec3 contribution = apply_light(u_lights[i], normalize(normal), world_pos);
        result += contribution;
        if (i == 0 && int(u_lights[i][0].w + 0.5) == MODEL_LIGHT_DIRECT)
            v_shadowlight = contribution - u_lights[i][3].rgb * u_lights[i][3].a;
    }
    return result;
}
vec4 vert() {
    vec4 position = vec4(0.0);
    vec4 normal = vec4(0.0);
    for (int i = 0; i < 4; ++i) {
        int bone_idx = int(a_skin1[i]) + int(u_first_bone_lookup_index);
        position += u_bones[bone_idx] * vec4(a_position, 1.0) * a_bone_weight1[i];
        normal += u_bones[bone_idx] * vec4(a_normal, 0.0) * a_bone_weight1[i];
    }
    position.w = 1.0;
    if (u_grass_params[3].z > 0.5) {
        float height = max(u_grass_params[3].y - u_grass_params[3].x, 0.001);
        float top = smoothstep(u_grass_params[1].w, 1.0, clamp((position.z - u_grass_params[3].x) / height, 0.0, 1.0));
        float phase = dot(a_instance[3].xy, u_grass_params[2].xy);
        position.xy += u_grass_params[2].zw * sin(u_grass_params[1].x * u_grass_params[1].y + phase) * u_grass_params[1].z * height * top;
    }
    vec4 world_pos = a_instance * position;
    v_color = a_color;
    if (u_grass_params[3].z > 0.5) {
        float fade_dist = length(world_pos.xy - u_grass_params[0].xy);
        v_color.a *= 1.0 - smoothstep(u_grass_params[0].z, u_grass_params[0].w, fade_dist);
    }
    v_texcoord = a_texcoord;
    v_texcoord2 = (u_texture_matrix * world_pos).xy;
    v_lighting = vertex_lighting(mat3(a_instance) * normal.xyz, world_pos.xyz);
    v_shadow = u_light_matrix * world_pos;
    return u_view_projection * world_pos;
}";

const MODEL_FRAGMENT_BODY: &str = "float get_fogofwar() { return texture(u_fog_of_war, v_texcoord2).r; }
float shadow_visibility(sampler2D map, vec4 shadow) {
    float depth = texture(map, vec2(shadow.x + 1.0, shadow.y + 1.0) * 0.5).r;
    return depth < (shadow.z + 0.99) * 0.5 ? 0.0 : 1.0;
}
vec4 frag() {
    vec2 uv = (u_uv_matrix * vec3(v_texcoord, 1.0)).xy;
    vec4 col = texture(u_texture, uv) * u_geoset_color * u_layer_alpha * v_color;
    if (!u_unshaded) {
        vec3 light = min(v_lighting - v_shadowlight * (1.0 - shadow_visibility(u_shadowmap, v_shadow)), vec3(1.0));
        col.rgb *= get_fogofwar() * light;
    }
    if (u_fog_enable) {
        float range = u_fog_params.y - u_fog_params.x;
        float depth = gl_FragCoord.z / gl_FragCoord.w;
        float factor = abs(range) > 0.0001 ? clamp((u_fog_params.y - depth) / range, 0.0, 1.0) : (depth <= u_fog_params.x ? 1.0 : 0.0);
        col.rgb = mix(u_fog_color, col.rgb, factor);
    }
    if (u_alpha_key && col.a < u_alpha_cutoff) discard;
    return col;
}";

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
    "const int MODEL_LIGHT_OMNI = 0;
const int MODEL_LIGHT_DIRECT = 1;
const int MODEL_LIGHT_AMBIENT = 2;
vec3 apply_light(mat4 light, vec3 n, vec3 world_pos) {
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
vec3 vertex_lighting(vec3 normal, vec3 world_pos) {
    vec3 n = normalize(normal);
    vec3 lighting = vec3(0.0);
    v_shadowlight = vec3(0.0);
    for (int i = 0; i < 8; ++i) {
        if (i >= u_light_count) break;
        vec3 contribution = apply_light(u_lights[i], n, world_pos);
        lighting += contribution;
        if (i == 0 && int(u_lights[i][0].w + 0.5) == MODEL_LIGHT_DIRECT)
            v_shadowlight = contribution - u_lights[i][3].rgb * u_lights[i][3].a;
    }
    return lighting;
}
vec4 vert() {
    vec4 pos4 = vec4(a_position, 1.0);
    vec4 norm4 = vec4(a_normal, 0.0);
    vec4 position = vec4(0.0);
    vec4 normal = vec4(0.0);
    for (int i = 0; i < 4; ++i) {
        int bone_idx = int(a_skin1[i]) + int(u_first_bone_lookup_index);
        position += u_bones[bone_idx] * pos4 * a_bone_weight1[i];
        normal += u_bones[bone_idx] * norm4 * a_bone_weight1[i];
    }
    position.w = 1.0;
    v_color = a_color;
    v_texcoord = a_texcoord;
    v_texcoord2 = (u_texture_matrix * u_model * position).xy;
    vec3 world_normal = normalize(u_normal_matrix * normal.xyz);
    vec3 world_pos = (u_model * position).xyz;
    v_lighting = vertex_lighting(world_normal, world_pos);
    v_shadow = u_light_matrix * u_model * position;
    return u_view_projection * u_model * position;
}",
    "float get_fogofwar() {
    return texture(u_fog_of_war, v_texcoord2).r;
}
float shadow_visibility(sampler2D shadowmap, vec4 shadow) {
    float depth = texture(shadowmap, vec2(shadow.x + 1.0, shadow.y + 1.0) * 0.5).r;
    return depth < (shadow.z + 0.99) * 0.5 ? 0.0 : 1.0;
}
vec4 frag() {
    vec2 uv = (u_uv_matrix * vec3(v_texcoord, 1.0)).xy;
    vec4 col = texture(u_texture, uv);
    col *= u_geoset_color;
    col *= u_layer_alpha;
    col *= v_color;
    if (!u_unshaded) {
        vec3 light = v_lighting;
        light -= v_shadowlight * (1.0 - shadow_visibility(u_shadowmap, v_shadow));
        light = min(light, vec3(1.0));
        col.rgb *= get_fogofwar() * light;
    }
    if (u_fog_enable) {
        float fog_range = u_fog_params.y - u_fog_params.x;
        float depth = gl_FragCoord.z / gl_FragCoord.w;
        float fog_factor = abs(fog_range) > 0.0001
            ? clamp((u_fog_params.y - depth) / fog_range, 0.0, 1.0)
            : (depth <= u_fog_params.x ? 1.0 : 0.0);
        col.rgb = mix(u_fog_color, col.rgb, fog_factor);
    }
    if (u_alpha_key) {
        if (col.a < u_alpha_cutoff) discard;
    }
    return col;
}",
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
    MODEL_INSTANCED_VERTEX_BODY,
    MODEL_FRAGMENT_BODY,
);
