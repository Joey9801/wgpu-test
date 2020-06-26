#version 450

layout(location = 0) in vec4 v_Color;
layout(location = 1) in vec3 v_Position;
layout(location = 2) in vec3 v_Normal;

layout(location = 0) out vec4 o_color;

layout(set = 0, binding = 0) uniform Locals {
    mat4 u_View;
    mat4 u_Proj;
};

const float screenGamma = 2.2; // Assume the monitor is calibrated to the sRGB color space

void main() {
    // NOTE: this position is in view space
    vec3 light_pos = (u_View * vec4(1.0, 4.0, 3.0, 1.0)).xyz;
    float light_power = 3.0;

    vec3 normal = normalize(v_Normal);

    vec3 light_dir = normalize(light_pos - v_Position);
    float light_distance = length(light_pos - v_Position);
    vec3 view_dir = normalize(-v_Position);
    vec3 half_dir = normalize(light_dir + view_dir);

    float lambertian = max(dot(light_dir, normal), 0.0);

    float spec_angle = max(dot(half_dir, normal), 0.0);
    float specular = pow(spec_angle, 15.0);

    vec3 colorLinear = vec3(0.02, 0.02, 0.02)
                     + v_Color.xyz * lambertian * vec3(1.0, 1.0, 1.0) * light_power / light_distance
                     + v_Color.xyz * specular * vec3(1.0, 1.0, 1.0) * light_power / light_distance;

    vec3 colorGammaCorrected = pow(colorLinear, vec3(1.0 / screenGamma));

    o_color = vec4(colorGammaCorrected, 1.0);
}