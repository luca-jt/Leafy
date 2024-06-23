#version 450 core

in vec4 v_color;
in vec2 v_uv;
in vec3 v_normal;
in float v_tex_idx;
in vec3 frag_pos;

out vec4 out_color;

uniform sampler2D tex_sampler[32];
uniform vec3 light_pos;

void main() {
    int sampler_idx = int(round(v_tex_idx));

    float ambient_light = 0.3;
    vec3 norm = normalize(v_normal);
    vec3 light_dir = normalize(light_pos - frag_pos);
    float diff = max(dot(norm, light_dir), 0.0);
    float light_strength = min(1.2, ambient_light + diff);

    out_color = texture(tex_sampler[sampler_idx], v_uv).rgba * v_color * light_strength;
}
