#version 150

uniform mat4 modelTransform;
uniform mat4 normalTransform;
uniform mat4 modelViewTransform;
uniform mat4 modelViewProjection;
uniform mat4 viewTransform;
uniform mat4 projectionTransform;

in vec4 vertexPosition;
in vec3 vertexNormal;

out vec4 worldPosition;
out vec4 viewPosition;
out vec3 viewNormal;

void main(void)
{
    worldPosition = modelTransform * vertexPosition;
    viewPosition = modelViewTransform * vertexPosition;
    viewNormal = normalize(mat3(normalTransform) * vertexNormal);
    gl_Position = modelViewProjection * vertexPosition;
}
