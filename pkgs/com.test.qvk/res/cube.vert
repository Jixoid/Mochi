#version 450

layout(location = 0) in vec3 pos;
layout(location = 1) in vec3 color;
layout(location = 2) in vec3 normal;
layout(location = 3) in vec2 uv;
layout(location = 4) in mat4 model;

layout(location = 0) out vec3 fragColor;
layout(location = 1) out vec2 fragUV;
layout(location = 2) out vec3 fragPosWorld;
layout(location = 3) out vec3 fragNormalWorld;

layout(set = 0, binding = 0, row_major) uniform CameraBuffer {
    mat4 view;
    mat4 proj;
} camera;

void main() {
    fragColor = color;
    fragUV = uv;
    
    // Matris transpozunu (satır-sütun çevirmesi) yapıyoruz
    mat4 trueModel = transpose(model);
    vec4 worldPos = trueModel * vec4(pos, 1.0);
    
    // Fragment shader'a gidecek verileri gönderiyoruz
    fragPosWorld = worldPos.xyz;
    
    // Normal vektörleri, model döndüğünde bozulmasın diye Normal Matrisi uyguluyoruz
    mat3 normalMatrix = transpose(inverse(mat3(trueModel)));
    fragNormalWorld = normalize(normalMatrix * normal);
    
    // Ekranda çizeceği nihai konumu veriyoruz
    gl_Position = camera.proj * camera.view * worldPos;
}