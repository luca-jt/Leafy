#version 450 core

in vec4 v_color;
in vec2 v_uv;
in vec3 v_normal;
in vec3 frag_pos;
in vec4 frag_pos_light[5];

out vec4 out_color;

struct LightData {
    vec4 light_pos;
    mat4 light_matrix;
};

layout (std140, binding = 0, column_major) uniform light_data {
    LightData lights[5];
};

uniform sampler2D shadow_sampler[5];
uniform sampler2D tex_sampler;
uniform int num_lights;

float shadow_calc(vec4 fpl) {
    vec3 proj_coords = fpl.xyz / fpl.w;
    proj_coords = proj_coords * 0.5 + 0.5;

    float bias = 0.001;
    int filter_size = 2;

    float shadow = 0.0;

    for (int i = 0; i < num_lights; i++) {
        for (int y = -filter_size / 2; y < filter_size / 2; ++y) {
            for (int x = -filter_size / 2; x < filter_size / 2; ++x) {
                vec2 offset = vec2(x, y) / textureSize(shadow_sampler[i], 0);
                float depth = texture(shadow_sampler[i], proj_coords.xy + offset).x;
                shadow += proj_coords.z > depth + bias ? 1.0 : 0.0;
            }
        }
    }
    shadow /= float(pow(filter_size, 2) * num_lights);

    if (proj_coords.z > 1.0) {
        shadow = 0.0;
    }

    return shadow;
}

void main() {
    float ambient_light = 0.3;
    vec3 norm = normalize(v_normal);

    float light_strength = 0.0;
    for (int i = 0; i < num_lights; i++) {
        vec3 light_dir = normalize(lights[i].light_pos.xyz - frag_pos);
        float diff = max(dot(norm, light_dir), 0.0);
        light_strength += min(1.2, ambient_light + diff * (1.0 - shadow_calc(frag_pos_light[i]))) / float(num_lights);
    }
    light_strength = light_strength > 0.0 ? light_strength : ambient_light;

    vec4 textured = texture(tex_sampler, v_uv).rgba * v_color;
    out_color = vec4(textured.rgb * light_strength, textured.a);
}
