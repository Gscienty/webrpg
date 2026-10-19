vec4 frag() {
    float f = 2.0 * abs(v_texcoord.x - 0.5);
    float k = smoothstep(0.0, 0.2, v_texcoord.y);
    return vec4(mix(1.0, f * f, k));
}
