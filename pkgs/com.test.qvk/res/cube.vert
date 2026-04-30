#version 450

layout(location = 0) in vec3 inPosition;
layout(location = 1) in vec3 inColor;
layout(location = 2) in vec2 inUV;

layout(location = 0) out vec3 fragColor;

layout(push_constant) uniform Constants {
    mat4 m;
} push;



void main() {
    fragColor = inColor;

    gl_Position = vec4(inPosition, 1.0) * push.m;
}
