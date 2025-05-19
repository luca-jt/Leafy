#version 450 core

in vec2 v_uv;
in vec4 v_color;

layout(location = 0) out vec4 out_color;
layout(location = 1) out vec4 bright_color;

layout(location = 0) uniform vec4 color;
layout(location = 1) uniform sampler2D tex_sampler;
layout(location = 2) uniform bool transparent_pass;

void main() {
    vec4 textured = texture(tex_sampler, v_uv).rgba * color * v_color;
    if (textured.a < 0.001 || (textured.a < 0.999 && !transparent_pass)) {
        discard;
    }
    out_color = vec4(textured.rgb + vec3(1.0), textured.a);
    bright_color = dot(out_color.rgb, vec3(0.2126, 0.7152, 0.0722)) > 1.001 ? vec4(out_color.rgb, 1.0) : vec4(0.0, 0.0, 0.0, 1.0);
}
