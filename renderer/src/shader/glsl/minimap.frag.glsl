vec4 frag() {
    float mask = 1.0 - smoothstep(0.49, 0.5, length(v_color.rg - vec2(0.5)));
    vec4 tex = texture(u_texture, v_texcoord);
    return vec4(tex.rgb, tex.a * mask);
}
