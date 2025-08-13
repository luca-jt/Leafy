#version 450 core

in vec2 v_uv;
in vec4 v_color;

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

layout(location = 0) uniform vec4 color;
layout(location = 1) uniform sampler2D tex_sampler;
layout(location = 21) uniform bool is_light_source; // this is only relevant in this shader

void main() {
    vec4 textured = texture(tex_sampler, v_uv).rgba * color * v_color;
    if (textured.a < 0.001) {
        discard;
    }

    float bloom_threshold = 1.0001 + bloom_threshold_shift;
    float bright_diff = bloom_threshold / ((textured.r * 0.2126 + textured.g * 0.7152 + textured.b * 0.0722) * 3.0) + 0.001;
    float added_brightness = is_light_source && bright_diff > 0.0 ? bright_diff : 0.0;

    out_color = vec4(textured.rgb + vec3(added_brightness), textured.a);
    bright_color = dot(out_color.rgb, vec3(0.2126, 0.7152, 0.0722)) > bloom_threshold ? vec4(out_color.rgb, 1.0) : vec4(0.0, 0.0, 0.0, 1.0);
}
