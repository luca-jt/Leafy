#version 450 core

in vec2 tex_coords_v;

layout(location = 0) out vec4 out_color;

layout (std140, binding = 3, column_major) uniform post_process {
    float gamma;
    float hue;
    float saturation;
    float value;
    float exposure;
};

layout(location = 0) uniform sampler2D scene_texture;
layout(location = 1) uniform sampler2D bloom_texture;

vec3 rgb2hsv(vec3 c) {
    vec4 K = vec4(0.0, -1.0 / 3.0, 2.0 / 3.0, -1.0);
    vec4 p = mix(vec4(c.bg, K.wz), vec4(c.gb, K.xy), step(c.b, c.g));
    vec4 q = mix(vec4(p.xyw, c.r), vec4(c.r, p.yzx), step(p.x, c.r));
    float d = q.x - min(q.w, q.y);
    float e = 1.0e-10;
    return vec3(abs(q.z + (q.w - q.y) / (6.0 * d + e)), d / (q.x + e), q.x);
}

vec3 hsv2rgb(vec3 c) {
    vec4 K = vec4(1.0, 2.0 / 3.0, 1.0 / 3.0, 3.0);
    vec3 p = abs(fract(c.xxx + K.xyz) * 6.0 - K.www);
    return c.z * mix(K.xxx, clamp(p - K.xxx, 0.0, 1.0), c.y);
}

vec3 clamp_vec(vec3 v, float min_val, float max_val) {
    float x = clamp(v.x, min_val, max_val);
    float y = clamp(v.y, min_val, max_val);
    float z = clamp(v.z, min_val, max_val);
    return vec3(x, y, z);
}

void main() {
    vec4 textured = texture(scene_texture, tex_coords_v);
    if (textured.a < 0.001) {
        discard;
    }
    vec3 bloom_color = texture(bloom_texture, tex_coords_v).rgb;

    // hdr tone mapping
    vec3 hdr_color = textured.rgb + bloom_color;
    vec3 tone_mapped = vec3(1.0) - exp(-hdr_color * exposure);

    // gamma correction
    vec3 corrected = pow(tone_mapped, vec3(1.0 / gamma));

    // changes in hsv
    vec3 hsv = rgb2hsv(clamp_vec(corrected, 0.0, 1.0));
    vec3 edited = vec3(hsv.x * hue, hsv.y * saturation, hsv.z * value);
    vec3 final_hsv = clamp_vec(edited, 0.0, 1.0);

    // conversion back to rgb
    vec3 final_rgb = hsv2rgb(final_hsv);
    out_color = vec4(final_rgb, textured.a);
}
