#version 450

layout(location = 0) in vec3 a_Position;
layout(location = 1) in vec3 a_Normal;
layout(location = 2) in vec4 a_Color;
layout(location = 3) in mat4 a_ModelMatrix;
layout(location = 7) in mat4 a_NormalMatrix;

layout(location = 0) out vec4 v_Color;
layout(location = 1) out vec3 v_Position;
layout(location = 2) out vec3 v_Normal;

layout(set = 0, binding = 0) uniform Locals {
    mat4 u_View;
    mat4 u_Proj;
};

void main() {
    v_Color = a_Color;
    v_Position = (u_View * a_ModelMatrix * vec4(a_Position, 1.0)).xyz;
    v_Normal = normalize(a_NormalMatrix * vec4(a_Normal, 1.0)).xyz;

    gl_Position = u_Proj * vec4(v_Position, 1.0);
}