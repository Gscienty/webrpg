vec4 frag() {
    float visibility = texture(u_texture, vec2(v_texcoord.x, 1.0 - v_texcoord.y)).r;
    float alpha = clamp(1.0 - visibility, 0.0, 1.0) * v_color.a;
    return vec4(v_color.rgb, alpha);
}
