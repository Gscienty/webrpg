float get_fogofwar() {
    return texture(u_fog_of_war, v_texcoord2).r;
}
float shadow_visibility(sampler2D depths, vec4 lightpos) {
    vec3 p = lightpos.xyz / lightpos.w * 0.5 + 0.5;
    if (any(lessThan(p, vec3(0.0))) || any(greaterThan(p, vec3(1.0)))) return 1.0;
    vec2 texel = 1.0 / vec2(textureSize(depths, 0));
    float lit = 0.0;
    for (int y = -1; y <= 1; y++) for (int x = -1; x <= 1; x++)
        lit += step(p.z - 0.0001, texture(depths, p.xy + vec2(x, y) * texel).r);
    return lit / 9.0;
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
}
