#version 450

layout(location = 0) in vec2 a_ScreenTopLeft;
layout(location = 1) in vec2 a_ScreenSize;
layout(location = 2) in vec2 a_AtlasTopLeft;
layout(location = 3) in vec2 a_AtlasSize;

layout(location = 0) out vec2 v_AtlasCoord;

void main() {
    vec2 screenCoord;
    switch (gl_VertexIndex) {
        case 0:
            screenCoord = a_ScreenTopLeft;
            v_AtlasCoord = a_AtlasTopLeft;
            break;
        case 1:
            screenCoord = a_ScreenTopLeft + vec2(a_ScreenSize.x, 0);
            v_AtlasCoord = a_AtlasTopLeft + vec2(a_AtlasSize.x, 0);
            break;
        case 2:
            screenCoord = a_ScreenTopLeft + vec2(0, a_ScreenSize.y);
            v_AtlasCoord = a_AtlasTopLeft + vec2(0, a_AtlasSize.y);
            break;
        case 3:
            screenCoord = a_ScreenTopLeft + a_ScreenSize;
            v_AtlasCoord = a_AtlasTopLeft + a_AtlasSize;
            break;
        default:
            // Write outside of clip space to discard the vertex
            gl_Position = vec4(10.0, 10.0, 10.0, 1.0);
    }

    gl_Position = vec4(screenCoord, 0.0, 1.0);
}