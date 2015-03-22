#version 150

in vec4 fragPosition;

out vec4 fragmentColor;

void main(void)
{
    fragmentColor = vec4(fragPosition.xyz, 1.0);
}
