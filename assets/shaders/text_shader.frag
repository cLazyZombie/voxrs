#version 450

layout(location=0) in vec3 v_color;
layout(location=1) in vec2 v_uv;

layout(set = 1, binding = 0) uniform texture2D t_font_atlas;
layout(set = 1, binding = 1) uniform sampler s_font_atlas;

layout(location=0) out vec4 f_color;

void main() {
    f_color = vec4(v_color, 1.0) * texture(sampler2D(t_font_atlas, s_font_atlas), v_uv);
}