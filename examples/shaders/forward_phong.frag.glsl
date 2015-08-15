#version 150

uniform vec4 cameraPosition;
uniform vec4 lightPosition;
uniform vec4 globalAmbient;
uniform mat4 viewTransform;
uniform mat4 modelViewTransform;

in vec4 worldPosition;
in vec4 viewPosition;
in vec3 viewNormal;

out vec4 fragmentColor;

void main(void)
{
    // STUFF THAT NEEDS TO BECOME UNIFORMS
    vec4 surfaceDiffuse = vec4(1.0, 0.0, 1.0, 1.0);
    vec4 lightColor = vec4(1.0, 1.0, 1.0, 1.0);
    vec4 surfaceSpecular = vec4(1.0, 1.0, 1.0, 1.0);
    float surfaceShininess = 3.0;

    // Calculate phong illumination.
    vec4 ambient = vec4(0.0, 0.0, 0.0, 1.0);
    vec4 diffuse = vec4(0.0, 0.0, 0.0, 1.0);
    vec4 specular = vec4(0.0, 0.0, 0.0, 1.0);

    ambient = globalAmbient * surfaceDiffuse;

    vec3 N = normalize(viewNormal);
    vec3 L = normalize((lightPosition - viewPosition).xyz);
    vec3 V = normalize(-viewPosition.xyz);

    float LdotN = dot(L, N);

    diffuse += surfaceDiffuse * lightColor * max(LdotN, 1e-6);

    vec3 R = normalize(reflect(-L, N));

    if (LdotN > 1e-6)
    {
        float RdotV = dot(R, V);
        specular = surfaceSpecular * lightColor * pow(RdotV, surfaceShininess);
    }

    fragmentColor = ambient + diffuse + specular;
}
