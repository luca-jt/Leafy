#version 450 core

in vec4 v_outline_color;

layout(location = 0) out vec4 out_color;
layout(location = 1) out vec4 bright_color;

void main() {
    if (v_outline_color.a < 0.001) {
        discard;
    }

    out_color = v_outline_color;
    bright_color = vec4(0.0, 0.0, 0.0, 1.0);
}
