vec4 vert() {
    vec4 pos = vec4(a_position, 1.0);
    vec3 eye = vec3(u_eye_position, 0.0);
    vec3 up = vec3(0.0, 0.0, 1.0);
    vec3 dir = normalize(a_position - eye);
    vec3 side = normalize(cross(dir, up));
    if (distance(eye.xy, a_position.xy) > 100.0) {
        float x = (a_texcoord.x - 0.5) * 2.0;
        float y = a_texcoord.y;
        pos.xy += side.xy * x * 100.0;
        pos.xy += normalize(pos.xyz - eye).xy * y * 2000.0;
    }
    pos.z = 0.0;
    v_color = a_color;
    v_texcoord = a_texcoord;
    return u_view_projection * u_model * pos;
}
