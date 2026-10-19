vec4 vert() {
    mat4 m = u_view_projection;
    vec3 camera_left = normalize(vec3(m[0][0], m[1][0], m[2][0]));
    vec3 left = camera_left * a_size;
    vec3 up = normalize(vec3(m[0][1], m[1][1], m[2][1])) * a_size;
    vec3 pos;
    if (dot(a_tail, a_tail) > 0.0) {
        vec3 point = a_position - a_tail * (1.0 - a_axis.y);
        vec3 side = cross(normalize(a_tail), u_eye - point);
        float side_length = length(side);
        if (side_length < 0.0001) side = camera_left; else side /= side_length;
        pos = point + side * ((a_axis.x - 0.5) * a_size);
    } else {
        mat3 billboard = mat3(left, up, a_position);
        pos = billboard * vec3(a_axis - vec2(0.5), 1.0);
    }
    v_color = a_color;
    v_texcoord = a_texcoord;
    v_texcoord2 = (u_texture_matrix * vec4(pos, 1.0)).xy;
    return u_view_projection * vec4(pos, 1.0);
}
