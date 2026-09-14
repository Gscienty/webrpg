vec4 frag() {
    return texture(u_texture, v_texcoord) * v_color;
}
