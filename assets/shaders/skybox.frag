#version 450 core

in vec3 tex_coords_v; // 3D texture coordinate

layout(location = 0) out vec4 out_color;
layout(location = 1) out vec4 bright_color;

layout (std140, binding = 3, column_major) uniform post_process {
    float gamma;
    float hue;
    float saturation;
    float value;
    float exposure;
    bool use_bloom;
    float bloom_threshold_shift;
};

layout(location = 0) uniform samplerCube cube_map;
layout(location = 1) uniform float bloom_threshold_shift_skybox;

void main() {
    vec4 textured = texture(cube_map, tex_coords_v);
    if (textured.a < 0.001) {
        discard;
    }
    out_color = textured;
    float bloom_threshold = 1.0001 + bloom_threshold_shift + bloom_threshold_shift_skybox;
    bright_color = dot(out_color.rgb, vec3(0.2126, 0.7152, 0.0722)) > bloom_threshold ? vec4(out_color.rgb, 1.0) : vec4(0.0, 0.0, 0.0, 1.0);
}
