#version 450 core

in vec3 v_color;
in vec2 v_uv;
in vec3 v_normal;
in vec3 frag_pos;

out vec3 out_color;

uniform sampler2D tex_sampler;
uniform vec3 light_pos;

void main() {
    float ambient_light = 0.3;
    vec3 norm = normalize(v_normal);
    vec3 light_dir = normalize(light_pos - frag_pos);
    float diff = max(dot(norm, light_dir), 0.0);
    float light_strength = min(1.2, ambient_light + diff);

    out_color = texture(tex_sampler, v_uv).rgb * v_color * light_strength;
}
