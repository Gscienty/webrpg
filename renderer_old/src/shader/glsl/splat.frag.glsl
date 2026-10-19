float crop_edges(vec2 tc) {
    return step(abs(tc.x - 0.5), 0.5) * step(abs(tc.y - 0.5), 0.5);
}
vec4 frag() {
    vec4 col = texture(u_texture, v_texcoord) * v_color;
    col.a *= crop_edges(v_texcoord);
    return col;
}
