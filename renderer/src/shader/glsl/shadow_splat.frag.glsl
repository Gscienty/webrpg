float crop_edges(vec2 tc) {
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
}
