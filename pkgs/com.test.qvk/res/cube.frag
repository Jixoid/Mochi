#version 450

layout(location = 0) in vec3 fragColor;
layout(location = 1) in vec2 fragUV;
layout(location = 2) in vec3 fragPosWorld;
layout(location = 3) in vec3 fragNormalWorld;

layout(location = 0) out vec4 outColor;

// Vertex Shader'da yaptığın gibi Kamerayı buraya da alıyoruz
layout(set = 0, binding = 0, row_major) uniform CameraBuffer {
    mat4 view;
    mat4 proj;
} camera;

const float PI = 3.14159265359;

// --- PBR FONKSİYONLARI ---
float DistributionGGX(vec3 N, vec3 H, float roughness) {
    float a = roughness * roughness;
    float a2 = a * a;
    float NdotH = max(dot(N, H), 0.0);
    float NdotH2 = NdotH * NdotH;
    float nom   = a2;
    float denom = (NdotH2 * (a2 - 1.0) + 1.0);
    denom = PI * denom * denom;
    return nom / max(denom, 0.0000001);
}

float GeometrySchlickGGX(float NdotV, float roughness) {
    float r = (roughness + 1.0);
    float k = (r * r) / 8.0;
    float nom   = NdotV;
    float denom = NdotV * (1.0 - k) + k;
    return nom / denom;
}

float GeometrySmith(vec3 N, vec3 V, vec3 L, float roughness) {
    float NdotV = max(dot(N, V), 0.0);
    float NdotL = max(dot(N, L), 0.0);
    float ggx2 = GeometrySchlickGGX(NdotV, roughness);
    float ggx1 = GeometrySchlickGGX(NdotL, roughness);
    return ggx1 * ggx2;
}

vec3 FresnelSchlick(float cosTheta, vec3 F0) {
    return F0 + (1.0 - F0) * pow(clamp(1.0 - cosTheta, 0.0, 1.0), 5.0);
}

void main() {
    // --- MATERYAL AYARLARI ---
    vec3 albedo     = fragColor; 
    float metallic  = 0.2;       
    float roughness = 0.3;       
    float ao        = 1.0;       

    // --- IŞIK KAYNAĞI ---
    vec3 lightPos   = vec3(5.0, 10.0, 5.0);
    vec3 lightColor = vec3(300.0, 300.0, 300.0); 

    vec3 N = normalize(fragNormalWorld);
    
    // Kameranın dünya uzayındaki pozisyonunu view matrisini tersine çevirerek alıyoruz (row_major layout'una uygun)
    mat4 trueView = transpose(camera.view);
    vec3 camPos = (inverse(trueView) * vec4(0.0, 0.0, 0.0, 1.0)).xyz;
    vec3 V = normalize(camPos - fragPosWorld);

    vec3 F0 = vec3(0.04); 
    F0 = mix(F0, albedo, metallic);

    // Işık hesaplamaları
    vec3 L = normalize(lightPos - fragPosWorld);
    vec3 H = normalize(V + L);
    
    float distance = length(lightPos - fragPosWorld);
    float attenuation = 1.0 / (distance * distance); 
    vec3 radiance = lightColor * attenuation;

    // BRDF
    float NDF = DistributionGGX(N, H, roughness);   
    float G   = GeometrySmith(N, V, L, roughness);      
    vec3 F    = FresnelSchlick(max(dot(H, V), 0.0), F0);       
    
    vec3 numerator    = NDF * G * F;
    float denominator = 4.0 * max(dot(N, V), 0.0) * max(dot(N, L), 0.0) + 0.0001;
    vec3 specular     = numerator / denominator;
    
    vec3 kS = F;
    vec3 kD = vec3(1.0) - kS;
    kD *= 1.0 - metallic;

    float NdotL = max(dot(N, L), 0.0);        
    vec3 Lo = (kD * albedo / PI + specular) * radiance * NdotL;

    vec3 ambient = vec3(0.03) * albedo * ao;
    vec3 color = ambient + Lo;

    // HDR & Gamma
    color = color / (color + vec3(1.0)); 
    color = pow(color, vec3(1.0/2.2));   

    outColor = vec4(color, 1.0);
}