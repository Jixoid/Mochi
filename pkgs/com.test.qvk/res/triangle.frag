#version 450

layout(location = 0) in vec3 fragColor;
layout(location = 0) out vec4 outColor;

void main() {
    // Vertex shader'dan gelen rengi ekrana bas (Alfa kanalı 1.0)
    outColor = vec4(fragColor, 1.0);
}
