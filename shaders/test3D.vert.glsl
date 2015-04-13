#version 150

uniform mat4 modelTransform;
uniform mat4 viewTransform;

in vec4 vertexPosition;

out vec4 fragPosition;

void main(void)
{
    fragPosition = vertexPosition + 0.5;
    gl_Position = viewTransform * modelTransform * vertexPosition;
}
