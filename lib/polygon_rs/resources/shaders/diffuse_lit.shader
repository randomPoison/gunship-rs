program vert {
    #version 150

    uniform mat4 modelTransform;
    uniform mat4 normalTransform;
    uniform mat4 modelViewTransform;
    uniform mat4 modelViewProjection;

    in vec4 vertexPosition;
    in vec3 vertexNormal;

    out vec4 viewPosition;
    out vec3 viewNormal;

    void main(void) {
        viewPosition = modelViewTransform * vertexPosition;
        viewNormal = normalize(mat3(normalTransform) * vertexNormal);
        gl_Position = modelViewProjection * vertexPosition;
    }
}

program frag {
    #version 150

    uniform vec4 globalAmbient;

    uniform vec4 lightPosition;
    uniform float lightStrength;
    uniform float lightRadius;
    uniform vec4 lightColor;

    // TODO: Make this a material property!
    uniform vec4 surfaceDiffuse;
    uniform vec4 surfaceSpecular;
    uniform float surfaceShininess;

    in vec4 viewPosition;
    in vec3 viewNormal;

    out vec4 fragmentColor;

    void main(void) {
        // Calculate phong illumination.
        vec4 ambient = vec4(0.0, 0.0, 0.0, 1.0);
        vec4 diffuse = vec4(0.0, 0.0, 0.0, 1.0);
        vec4 specular = vec4(0.0, 0.0, 0.0, 1.0);

        ambient = globalAmbient * surfaceDiffuse;

        vec3 lightOffset = (lightPosition - viewPosition).xyz;
        float dist = length(lightOffset);

        vec3 N = normalize(viewNormal);
        vec3 L = normalize(lightOffset);
        vec3 V = normalize(-viewPosition.xyz);

        float LdotN = dot(L, N);
        float attenuation = 1.0 / pow((dist / lightRadius) + 1.0, 2.0);

        diffuse += surfaceDiffuse * lightColor * max(LdotN, 1.0e-6) * attenuation * lightStrength;

        // Apply specular.
        if (LdotN > 1e-6) {
            vec3 R = normalize(reflect(-L, N));
            float RdotV = clamp(dot(R, V), 0.0, 1.0);
            specular = surfaceSpecular * lightColor * pow(RdotV, surfaceShininess) * attenuation * lightStrength;
        }

        fragmentColor = ambient + diffuse + specular;
    }
}
