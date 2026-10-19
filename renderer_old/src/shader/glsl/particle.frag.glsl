vec4 frag() {
    vec4 col = texture(u_texture, v_texcoord) * v_color;
    col.rgb *= texture(u_fog_of_war, v_texcoord2).r;
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
