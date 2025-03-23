#version 450 core

in vec2 tex_coords_v;

out vec4 out_color;

layout(location = 0) uniform sampler2D tex_sampler;

void main() {
    vec4 textured = texture(tex_sampler, tex_coords_v);
    if (textured.a < 0.001) {
        discard;
    }
    //float gamma = 2.2;
    float gamma = 1.0;
    out_color = vec4(pow(textured.rgb, vec3(1.0 / gamma)), textured.a);
}
