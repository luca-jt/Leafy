#version 450 core

in vec2 tex_coords_v;

out vec4 out_color;

layout(location = 0) uniform sampler2D tex_sampler;
layout(location = 1) uniform float gamma;
layout(location = 2) uniform float hue;
layout(location = 3) uniform float saturation;
layout(location = 4) uniform float value;

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
    vec4 textured = texture(tex_sampler, tex_coords_v);
    if (textured.a < 0.001) {
        discard;
    }
    vec4 rendered = vec4(pow(textured.rgb, vec3(1.0 / gamma)), textured.a);

    // changes in hsv
    vec3 hsv = rgb2hsv(clamp_vec(rendered.rgb, 0.0, 1.0));
    vec3 edited = vec3(hsv.x * hue, hsv.y * saturation, hsv.z * value);
    vec3 final_hsv = clamp_vec(edited, 0.0, 1.0);

    // conversion back to rgb
    vec3 final_rgb = hsv2rgb(final_hsv);
    out_color = vec4(final_rgb, rendered.a);
}
