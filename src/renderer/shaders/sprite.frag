#version 450

layout(location = 0) in vec2 v_AtlasCoord;

layout(set = 0, binding = 0) uniform texture2D t_atlas;
layout(set = 0, binding = 1) uniform sampler s_atlas;

layout(location = 0) out vec4 o_color;

void main() {
    o_color = texture(sampler2D(t_atlas, s_atlas), v_AtlasCoord);
}