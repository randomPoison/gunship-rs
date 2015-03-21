#version 150

in vec4 vertexPosition;

out vec4 fragPosition;

void main(void)
{
    fragPosition = vertexPosition;
    gl_Position = vec4(vertexPosition.xyz * 0.5, 1.0);
}
