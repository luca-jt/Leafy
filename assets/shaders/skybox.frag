#version 450 core

in vec3 tex_coords_v; // 3D texture coordinate

out vec4 out_color;

layout(location = 0) uniform samplerCube cube_map;

void main() {
    out_color = texture(cube_map, tex_coords_v);
}
