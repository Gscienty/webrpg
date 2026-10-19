vec4 vert() {
    v_texcoord = a_texcoord;
    v_color = a_color;
    return u_view_projection * u_model * vec4(a_position, 1.0);
}
