vec4 frag() {
    vec4 col = texture(u_texture, v_texcoord) * v_color;
    float glow = max(abs(v_texcoord.x - 0.5), abs(v_texcoord.y - 0.5));
    glow = smoothstep(0.33, 0.5, glow) * 0.75 * u_active_glow;
    col.rgb = mix(col.rgb, vec3(0.5, 1.0, 0.5), glow);
    vec2 radial = v_texcoord - vec2(0.5);
    float angle = atan(radial.x, -radial.y) / 6.28318530718;
    angle = angle < 0.0 ? angle + 1.0 : angle;
    float elapsed = 1.0 - clamp(u_radial_shade, 0.0, 1.0);
    float shade = step(elapsed, angle) * step(0.000001, u_radial_shade);
    col.rgb *= mix(1.0, 0.35, shade);
    float crop = step(abs(v_texcoord.x - 0.5), 0.5) * step(abs(v_texcoord.y - 0.5), 0.5);
    col.a *= crop;
    return col;
}
