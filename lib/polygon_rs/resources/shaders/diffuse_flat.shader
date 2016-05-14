program vert {
    uniform mat4 modelViewProjection;

    in vec4 vertexPosition;

    void main() {
        gl_Position = modelViewProjection * vertexPosition;
    }
}

program frag {
    uniform vec4 surfaceDiffuse;

    out vec4 colorOut;

    void main() {
        colorOut = surfaceDiffuse;
    }
}
