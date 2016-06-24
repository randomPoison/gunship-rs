program vert {
    uniform mat4 model_view_projection;

    in vec4 vertex_position;

    void main() {
        gl_Position = model_view_projection * vertex_position;
    }
}

program frag {
    uniform vec4 surface_color;

    out vec4 colorOut;

    void main() {
        colorOut = surface_color;
    }
}
