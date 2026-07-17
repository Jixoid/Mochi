/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#version 450

#extension GL_EXT_buffer_reference : require
#extension GL_EXT_shader_explicit_arithmetic_types_int64 : require


layout(location = 0) out vec4 out_color;

layout(location = 0) in vec3 pos_world;
layout(location = 1) in vec3 normal_world;

#if defined(WITH_COLOR)
	layout(location = 2) in vec3 color;
#elif defined(WITH_TEXTURE)
	layout(location = 2) in vec2 uv;
#endif


layout(buffer_reference, std430, row_major) readonly buffer CameraBuffer {
	mat4 view;
	mat4 proj;
};


struct light_t {
	vec4 pos;
	vec4 color;
};

layout(buffer_reference, std430) readonly buffer LightBuffer {
	uvec4 a;
	uvec4 b;
	light_t s[32];
};

layout(push_constant) uniform PushConstant {
  mat4 model;
  uint64_t vertex_addr;
  CameraBuffer camera;
  LightBuffer  light;
  
  #if defined(WITH_MULTI_INST)
    uint64_t inst_addr;
  #endif

  uint texture_id;
} push;

#if defined(WITH_TEXTURE)
	layout(set = 1, binding = 0) uniform sampler2D texSampler;
#endif


const float METALLIC  = 0.0;
const float ROUGHNESS = 0.5;
const float AMBIENT   = 0.03;

const float PI = 3.14159265359;



float DistributionGGX(vec3 N, vec3 H, float roughness) {
	float a      = roughness * roughness;
	float a2     = a * a;
	float NdotH  = max(dot(N, H), 0.0);
	float denom  = (NdotH * NdotH * (a2 - 1.0) + 1.0);
	return a2 / (PI * denom * denom);
}

float GeometrySchlickGGX(float NdotX, float roughness) {
	float r = roughness + 1.0;
	float k = (r * r) / 8.0;
	return NdotX / (NdotX * (1.0 - k) + k);
}

float GeometrySmith(vec3 N, vec3 V, vec3 L, float roughness) {
	return GeometrySchlickGGX(max(dot(N, V), 0.0), roughness) * GeometrySchlickGGX(max(dot(N, L), 0.0), roughness);
}

vec3 FresnelSchlick(float cosTheta, vec3 F0) {
	return F0 + (1.0 - F0) * pow(clamp(1.0 - cosTheta, 0.0, 1.0), 5.0);
}



void main() {
	vec4 albedo;
	#if defined(WITH_COLOR)
		albedo = vec4(color, 1.0);
	#elif defined(WITH_TEXTURE)
		albedo = texture(texSampler, uv);
	#endif


	vec3 N      = normalize(normal_world);
	vec3 camPos = -transpose(mat3(push.camera.view)) * push.camera.view[3].xyz;
	vec3 V      = normalize(camPos - pos_world);
	vec3 F0     = mix(vec3(0.04), albedo.xyz, METALLIC);

	vec3 Lo = vec3(0.0);

	int activeLightCount = int(push.light.a.x);
	for(int i = 0; i < activeLightCount; i++) {
		vec3  lightPos       = push.light.s[i].pos.xyz;
		vec3  lightCol       = push.light.s[i].color.xyz;
		float lightIntensity = push.light.s[i].color.w;

		vec3  L           = normalize(lightPos - pos_world);
		vec3  H           = normalize(V + L);
		float dist        = length(lightPos - pos_world);
		vec3  radiance    = lightCol * lightIntensity / (dist * dist);

		float NDF      = DistributionGGX(N, H, ROUGHNESS);
		float G        = GeometrySmith(N, V, L, ROUGHNESS);
		vec3  F        = FresnelSchlick(max(dot(H, V), 0.0), F0);

		vec3  specular = (NDF * G * F) / (4.0 * max(dot(N, V), 0.0) * max(dot(N, L), 0.0) + 0.0001);
		vec3 kD = (1.0 - F) * (1.0 - METALLIC);

		Lo += (kD * albedo.xyz / PI + specular) * radiance * max(dot(N, L), 0.0);
	}

	vec3 color = AMBIENT * albedo.xyz + Lo;
	color = color / (color + vec3(1.0));
	color = pow(color, vec3(1.0 / 2.2));

	out_color = vec4(color, 1.0);
}
