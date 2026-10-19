const int MODEL_LIGHT_OMNI = 0;
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
}
