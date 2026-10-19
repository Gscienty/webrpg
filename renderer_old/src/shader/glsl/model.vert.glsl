const int MODEL_LIGHT_OMNI = 0;
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
        if (i >= u_lights_count) break;
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
}
