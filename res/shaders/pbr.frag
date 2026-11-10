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
#extension GL_EXT_scalar_block_layout : require


struct DirectionalLight {
	vec4 direction;
	vec4 color;
};

struct PointLight {
	vec4 position;
	vec4 color;
};

layout(buffer_reference, scalar, row_major) readonly buffer SceneFrameData {
	mat4 view;
	mat4 proj;
	mat4 view_proj;
	mat4 inv_view_proj;
	vec4 camera_position;
	vec4 ambient_color;
	uint dir_light_count;
	uint point_light_count;
	uint _pad0;
	uint _pad1;
	DirectionalLight dir_lights[4];
	PointLight point_lights[64];
};

struct PbrMaterialData {
	vec4 base_color;
	vec4 emissive;
	float roughness;
	float metallic;
	float normal_scale;
	float occlusion;
	float alpha_cutoff;
	uint albedo_texture_id;
	uint normal_texture_id;
	uint orm_texture_id;
};

layout(buffer_reference, scalar) readonly buffer MaterialDataBuffer {
	PbrMaterialData materials[];
};

layout(push_constant, scalar, row_major) uniform PushConstants {
	mat4 model;
	uint64_t scene_address;
	uint64_t vertex_address;
	uint64_t material_address;
	uint material_id;
	uint flags;
} pc;

layout(location = 0) in vec3 in_world_pos;
layout(location = 1) in vec3 in_normal;
layout(location = 2) in vec4 in_color;
layout(location = 3) in vec2 in_uv;

layout(location = 0) out vec4 out_color;

const float PI = 3.14159265359;


float distribution_ggx(vec3 N, vec3 H, float roughness) {
	float a = roughness * roughness;
	float a2 = a * a;
	float NdotH = max(dot(N, H), 0.0);
	float denom = (NdotH * NdotH * (a2 - 1.0) + 1.0);
	return a2 / (PI * denom * denom);
}

float geometry_schlick_ggx(float NdotV, float roughness) {
	float r = roughness + 1.0;
	float k = (r * r) / 8.0;
	return NdotV / (NdotV * (1.0 - k) + k);
}

float geometry_smith(vec3 N, vec3 V, vec3 L, float roughness) {
	float NdotV = max(dot(N, V), 0.0);
	float NdotL = max(dot(N, L), 0.0);
	float ggx2 = geometry_schlick_ggx(NdotV, roughness);
	float ggx1 = geometry_schlick_ggx(NdotL, roughness);
	return ggx1 * ggx2;
}

vec3 fresnel_schlick(float cos_theta, vec3 F0) {
	return F0 + (1.0 - F0) * pow(clamp(1.0 - cos_theta, 0.0, 1.0), 5.0);
}

vec3 tonemap_aces(vec3 x) {
	const float a = 2.51;
	const float b = 0.03;
	const float c = 2.43;
	const float d = 0.59;
	const float e = 0.14;
	return clamp((x * (a * x + b)) / (x * (c * x + d) + e), 0.0, 1.0);
}

vec3 linear_to_srgb(vec3 color) {
	return pow(color, vec3(1.0 / 2.2));
}


void main() {
	SceneFrameData scene = SceneFrameData(pc.scene_address);
	MaterialDataBuffer mat_buf = MaterialDataBuffer(pc.material_address);
	PbrMaterialData mat = mat_buf.materials[pc.material_id];

	vec4 albedo = mat.base_color * in_color;
	float metallic = clamp(mat.metallic, 0.0, 1.0);
	float roughness = clamp(mat.roughness, 0.04, 1.0);

	vec3 N = normalize(in_normal);
	vec3 V = normalize(scene.camera_position.xyz - in_world_pos);

	vec3 F0 = vec3(0.04);
	F0 = mix(F0, albedo.rgb, metallic);

	vec3 Lo = vec3(0.0);

	// Evaluate directional lights
	uint dir_count = min(scene.dir_light_count, 4u);
	for (uint i = 0u; i < dir_count; ++i) {
		vec3 L = normalize(-scene.dir_lights[i].direction.xyz);
		vec3 H = normalize(V + L);
		vec3 radiance = scene.dir_lights[i].color.rgb * scene.dir_lights[i].color.w;

		float NDF = distribution_ggx(N, H, roughness);
		float G = geometry_smith(N, V, L, roughness);
		vec3 F = fresnel_schlick(max(dot(H, V), 0.0), F0);

		vec3 numerator = NDF * G * F;
		float denominator = 4.0 * max(dot(N, V), 0.0) * max(dot(N, L), 0.0) + 0.0001;
		vec3 specular = numerator / denominator;

		vec3 kS = F;
		vec3 kD = (vec3(1.0) - kS) * (1.0 - metallic);

		float NdotL = max(dot(N, L), 0.0);
		Lo += (kD * albedo.rgb / PI + specular) * radiance * NdotL;
	}

	// Evaluate point lights
	uint pt_count = min(scene.point_light_count, 64u);
	for (uint i = 0u; i < pt_count; ++i) {
		vec3 light_vec = scene.point_lights[i].position.xyz - in_world_pos;
		float distance = length(light_vec);
		float radius = scene.point_lights[i].position.w;

		if (distance > radius) continue;

		vec3 L = normalize(light_vec);
		vec3 H = normalize(V + L);

		// Smooth windowed attenuation
		float atten = clamp(1.0 - pow(distance / radius, 4.0), 0.0, 1.0);
		atten = (atten * atten) / (distance * distance + 1.0);

		vec3 radiance = scene.point_lights[i].color.rgb * scene.point_lights[i].color.w * atten;

		float NDF = distribution_ggx(N, H, roughness);
		float G = geometry_smith(N, V, L, roughness);
		vec3 F = fresnel_schlick(max(dot(H, V), 0.0), F0);

		vec3 numerator = NDF * G * F;
		float denominator = 4.0 * max(dot(N, V), 0.0) * max(dot(N, L), 0.0) + 0.0001;
		vec3 specular = numerator / denominator;

		vec3 kS = F;
		vec3 kD = (vec3(1.0) - kS) * (1.0 - metallic);

		float NdotL = max(dot(N, L), 0.0);
		Lo += (kD * albedo.rgb / PI + specular) * radiance * NdotL;
	}

	vec3 ambient = scene.ambient_color.rgb * albedo.rgb;
	vec3 color = ambient + Lo + mat.emissive.rgb * mat.emissive.w;

	// Tonemapping and gamma correction for sRGB display
	color = tonemap_aces(color);
	color = linear_to_srgb(color);

	out_color = vec4(color, albedo.a);
}
