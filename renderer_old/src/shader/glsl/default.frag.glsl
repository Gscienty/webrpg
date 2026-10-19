float get_light() {
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
}
