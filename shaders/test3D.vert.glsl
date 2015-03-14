#version 150

in vec4 vertexPosition;

void main(void)
{
    gl_Position = vertexPosition * 0.5;
}
