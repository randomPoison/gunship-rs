#version 150

in vec4 fragPosition;

out vec4 fragmentColor;

void main(void)
{
    fragmentColor = abs(fragPosition);
}
