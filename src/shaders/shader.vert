#version 450

layout(location = 0) in vec3 a_Pos;
layout(location = 1) in vec4 a_Color;
layout(location = 2) in mat4 a_model;

layout(location = 0) out vec4 o_Color;

layout(set = 0, binding = 0) uniform Locals {
    mat4 u_Transform;
};

void main() {
    gl_Position = u_Transform * a_model * vec4(a_Pos, 1.0);
    o_Color = a_Color;
}