property surface_diffuse: Texture2d;
property surface_color: Color;
property surface_specular: Color;
property surface_shininess: f32;

program frag {
    // Calculate phong illumination.
    vec4 ambient = vec4(0.0, 0.0, 0.0, 1.0);
    vec4 diffuse = vec4(0.0, 0.0, 0.0, 1.0);
    vec4 specular = vec4(0.0, 0.0, 0.0, 1.0);

    vec4 surface_diffuse_sampled = texture(surface_diffuse, @vertex.uv0) + surface_color;

    ambient = global_ambient * surface_diffuse_sampled;

    vec3 light_offset = (light_position - @vertex.view_position).xyz;
    float dist = length(light_offset);

    vec3 n = normalize(@vertex.view_normal);
    vec3 l = normalize(light_offset);
    vec3 v = normalize(-@vertex.view_position.xyz);

    float l_dot_n = dot(l, n);
    float attenuation = 1.0 / pow((dist / light_radius) + 1.0, 2.0);

    diffuse += surface_diffuse_sampled * light_color * max(l_dot_n, 1.0e-6) * attenuation * light_strength;

    // Apply specular.
    if (l_dot_n > 1e-6) {
        vec3 r = normalize(reflect(-l, n));
        float r_dot_v = clamp(dot(r, v), 0.0, 1.0);
        specular =
            surface_diffuse_sampled *
            light_color *
            pow(r_dot_v, surface_shininess) *
            attenuation *
            light_strength;
    }

    @color = ambient + diffuse + specular;
}
