#version 150

uniform mat4 modelTransform;

in vec4 vertexPosition;

out vec4 fragPosition;

void main(void)
{
    fragPosition = vertexPosition;
    gl_Position = modelTransform * vec4(vertexPosition.xyz * 0.5, 1.0);
}
