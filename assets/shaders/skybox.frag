#version 450 core

in vec3 tex_coords_v; // 3D texture coordinate

layout(location = 0) out vec4 out_color;
layout(location = 1) out vec4 bright_color;

layout(location = 0) uniform samplerCube cube_map;

void main() {
    vec4 textured = texture(cube_map, tex_coords_v);
    if (textured.a < 0.001) {
        discard;
    }
    out_color = textured;
    bright_color = dot(out_color.rgb, vec3(0.2126, 0.7152, 0.0722)) > 1.001 ? vec4(out_color.rgb, 1.0) : vec4(0.0, 0.0, 0.0, 1.0);
}
