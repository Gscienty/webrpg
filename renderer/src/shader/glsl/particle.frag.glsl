vec4 frag() {
    vec4 texel;
    if (v_texcoord.z < 0.5) texel = texture(u_texture[0], v_texcoord.xy);
    else if (v_texcoord.z < 1.5) texel = texture(u_texture[1], v_texcoord.xy);
    else if (v_texcoord.z < 2.5) texel = texture(u_texture[2], v_texcoord.xy);
    else texel = texture(u_texture[3], v_texcoord.xy);
    vec4 col = texel * v_color;
    col.rgb *= texture(u_fog_of_war, v_texcoord2).r;
    if (u_modulate != 0) {
        float neutral = u_modulate == 2 ? 0.5 : 1.0;
        col.rgb = mix(vec3(neutral), col.rgb, v_color.a);
    }
    if (u_alpha_key) {
#ifndef BZ_USE_MSAA
        if (col.a < u_alpha_cutoff) discard;
#else
        float edge = max(fwidth(col.a), 1.0 / 255.0);
        col.a = smoothstep(u_alpha_cutoff - edge, u_alpha_cutoff + edge, col.a);
#endif
    }
    return col;
}
