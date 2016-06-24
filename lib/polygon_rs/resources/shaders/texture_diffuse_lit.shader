program vert {
    #version 150

    uniform mat4 model_transform;
    uniform mat4 normal_transform;
    uniform mat4 model_view_transform;
    uniform mat4 model_view_projection;

    in vec4 vertex_position;
    in vec3 vertex_normal;
    in vec2 vertex_uv0;

    out vec4 view_position;
    out vec3 view_normal;
    out vec2 frag_uv0;

    void main(void) {
        view_position = model_view_transform * vertex_position;
        view_normal = normalize(mat3(normal_transform) * vertex_normal);
        frag_uv0 = vertex_uv0;
        gl_Position = model_view_projection * vertex_position;
    }
}

program frag {
    #version 150

    uniform vec4 global_ambient;

    uniform vec4 light_position;
    uniform float light_strength;
    uniform float light_radius;
    uniform vec4 light_color;

    // TODO: Make this a material property!
    uniform sampler2D surface_diffuse;
    uniform vec4 surface_color;
    uniform vec4 surface_specular;
    uniform float surface_shininess;

    in vec4 view_position;
    in vec3 view_normal;
    in vec2 frag_uv0;

    out vec4 frag_color;

    void main(void) {
        // Calculate phong illumination.
        vec4 ambient = vec4(0.0, 0.0, 0.0, 1.0);
        vec4 diffuse = vec4(0.0, 0.0, 0.0, 1.0);
        vec4 specular = vec4(0.0, 0.0, 0.0, 1.0);

        vec4 surface_diffuse_sampled = texture(surface_diffuse, frag_uv0) + surface_color;

        ambient = global_ambient * surface_diffuse_sampled;

        vec3 light_offset = (light_position - view_position).xyz;
        float dist = length(light_offset);

        vec3 n = normalize(view_normal);
        vec3 l = normalize(light_offset);
        vec3 v = normalize(-view_position.xyz);

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

        frag_color = ambient + diffuse;// + specular;
    }
}
