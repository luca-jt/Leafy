#version 450 core

in vec2 tex_coords_v;

out vec4 out_color;

layout(location = 0) uniform sampler2D tex_sampler;

void main() {
    vec4 textured = texture(tex_sampler, tex_coords_v);
    if (textured.a < 0.001) {
        discard;
    }
    out_color = textured;
}
