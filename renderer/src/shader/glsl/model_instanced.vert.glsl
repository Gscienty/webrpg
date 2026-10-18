const int MODEL_LIGHT_DIRECT = 1;
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
        if (i >= u_lights_count) break;
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
}
