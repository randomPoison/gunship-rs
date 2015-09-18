program vert {
    #version 150

    uniform mat4 modelViewProjection;

    in vec4 vertexPosition;

    void main(void)
    {
        gl_Position = modelViewProjection * vertexPosition;
    }
}

program frag {
    #version 150

    uniform vec4 surfaceColor;

    out vec4 fragmentColor;

    void main(void)
    {
        fragmentColor = surfaceColor;
    }
}
