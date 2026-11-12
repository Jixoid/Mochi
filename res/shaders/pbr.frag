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
#extension GL_EXT_nonuniform_qualifier : require


layout(set = 0, binding = 0) uniform sampler2D u_textures[];


struct DirectionalLight {
	vec4 direction;
	vec4 color;
};

struct PointLight {
	vec4 position;
	vec4 color;
};

struct SpotLight {
	vec4 position_radius;
	vec4 color_intensity;
	vec4 direction_cos_inner;
	vec4 params;
};

struct ClusterRecord {
	uint point_offset;
	uint point_count;
	uint spot_offset;
	uint spot_count;
};

layout(buffer_reference, scalar) readonly buffer ClusterGridBuffer {
	ClusterRecord records[];
};

layout(buffer_reference, scalar) readonly buffer LightIndexBuffer {
	uint indices[];
};

layout(buffer_reference, scalar) readonly buffer PointLightBuffer {
	PointLight lights[];
};

layout(buffer_reference, scalar) readonly buffer SpotLightBuffer {
	SpotLight lights[];
};

layout(buffer_reference, scalar, row_major) readonly buffer SceneFrameData {
	mat4 view;
	mat4 proj;
	mat4 view_proj;
	mat4 inv_view_proj;
	vec4 camera_position;
	vec4 ambient_color;
	uint64_t cluster_grid_address;
	uint64_t point_light_index_list_address;
	uint64_t spot_light_index_list_address;
	uint64_t point_lights_address;
	uint64_t spot_lights_address;
	vec2 cluster_screen_size;
	float z_near;
	float z_far;
	uvec4 cluster_dims;
	uint dir_light_count;
	uint point_light_count;
	uint spot_light_count;
	uint _pad0;
	uint _pad1_0;
	uint _pad1_1;
	DirectionalLight dir_lights[4];
	mat4 cascade_view_proj[4];
	vec4 cascade_split_depths;
	uvec4 cascade_shadow_ids;
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
	uint emissive_texture_id;
	uint _pad0;
	uint _pad1;
	uint _pad2;
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
layout(location = 4) in vec4 in_tangent;

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

float sample_shadow_bilinear(uint tex_id, vec2 uv, float current_depth, float bias) {
	vec2 tex_size = vec2(2048.0);
	vec2 texel_pos = uv * tex_size - 0.5;
	vec2 base_coord = floor(texel_pos);
	vec2 f = fract(texel_pos);

	vec2 inv_size = 1.0 / tex_size;
	vec2 uv00 = (base_coord + 0.5) * inv_size;
	vec2 uv10 = uv00 + vec2(inv_size.x, 0.0);
	vec2 uv01 = uv00 + vec2(0.0, inv_size.y);
	vec2 uv11 = uv00 + inv_size;

	float d00 = texture(u_textures[nonuniformEXT(tex_id)], uv00).r;
	float d10 = texture(u_textures[nonuniformEXT(tex_id)], uv10).r;
	float d01 = texture(u_textures[nonuniformEXT(tex_id)], uv01).r;
	float d11 = texture(u_textures[nonuniformEXT(tex_id)], uv11).r;

	float s00 = (current_depth - bias > d00) ? 0.0 : 1.0;
	float s10 = (current_depth - bias > d10) ? 0.0 : 1.0;
	float s01 = (current_depth - bias > d01) ? 0.0 : 1.0;
	float s11 = (current_depth - bias > d11) ? 0.0 : 1.0;

	return mix(mix(s00, s10, f.x), mix(s01, s11, f.x), f.y);
}

float evaluate_csm_shadow(SceneFrameData scene, vec3 world_pos, vec3 N, float view_z) {
	int cascade_idx = -1;
	for (int c = 0; c < 4; ++c) {
		if (view_z < scene.cascade_split_depths[c]) {
			cascade_idx = c;
			break;
		}
	}
	if (cascade_idx < 0) {
		return 1.0;
	}

	uint shadow_tex_id = scene.cascade_shadow_ids[cascade_idx];

	// Normal offset bias scaled by cascade index to eliminate shadow acne
	float normal_bias = 0.005 * float(cascade_idx + 1);
	vec3 biased_pos = world_pos + N * normal_bias;

	vec4 shadow_clip = scene.cascade_view_proj[cascade_idx] * vec4(biased_pos, 1.0);
	vec3 proj_coords = shadow_clip.xyz / shadow_clip.w;

	vec2 shadow_uv = proj_coords.xy * 0.5 + 0.5;
	float current_depth = proj_coords.z;

	if (current_depth > 1.0 || shadow_uv.x < 0.0 || shadow_uv.x > 1.0 || shadow_uv.y < 0.0 || shadow_uv.y > 1.0) {
		return 1.0;
	}

	float bias = 0.0004;
	vec2 inv_size = vec2(1.0 / 2048.0);

	// 4-tap bilinear PCF filter (equivalent to continuous 16-tap filtered shadow)
	float shadow = 0.0;
	shadow += sample_shadow_bilinear(shadow_tex_id, shadow_uv + vec2(-0.75, -0.75) * inv_size, current_depth, bias);
	shadow += sample_shadow_bilinear(shadow_tex_id, shadow_uv + vec2( 0.75, -0.75) * inv_size, current_depth, bias);
	shadow += sample_shadow_bilinear(shadow_tex_id, shadow_uv + vec2(-0.75,  0.75) * inv_size, current_depth, bias);
	shadow += sample_shadow_bilinear(shadow_tex_id, shadow_uv + vec2( 0.75,  0.75) * inv_size, current_depth, bias);

	return shadow * 0.25;
}


void main() {
	SceneFrameData scene = SceneFrameData(pc.scene_address);
	MaterialDataBuffer mat_buf = MaterialDataBuffer(pc.material_address);
	PbrMaterialData mat = mat_buf.materials[pc.material_id];

	vec4 albedo = mat.base_color * in_color;
	if (mat.albedo_texture_id != 0xFFFFFFFFu) {
		vec4 tex_albedo = texture(u_textures[nonuniformEXT(mat.albedo_texture_id)], in_uv);
		albedo *= tex_albedo;
	}

	if (mat.alpha_cutoff > 0.001 && albedo.a < mat.alpha_cutoff) {
		discard;
	}

	float metallic = clamp(mat.metallic, 0.0, 1.0);
	float roughness = clamp(mat.roughness, 0.04, 1.0);
	float occlusion = mat.occlusion;

	if (mat.orm_texture_id != 0xFFFFFFFFu) {
		vec3 orm = texture(u_textures[nonuniformEXT(mat.orm_texture_id)], in_uv).rgb;
		occlusion *= orm.r;
		roughness *= orm.g;
		metallic *= orm.b;
		roughness = clamp(roughness, 0.04, 1.0);
		metallic = clamp(metallic, 0.0, 1.0);
	}

	vec3 N = normalize(in_normal);
	if (mat.normal_texture_id != 0xFFFFFFFFu && length(in_tangent.xyz) > 0.001) {
		vec3 T = normalize(in_tangent.xyz);
		T = normalize(T - dot(T, N) * N);
		vec3 B = cross(N, T) * in_tangent.w;
		mat3 TBN = mat3(T, B, N);

		vec3 raw_normal = texture(u_textures[nonuniformEXT(mat.normal_texture_id)], in_uv).rgb * 2.0 - 1.0;
		raw_normal.xy *= mat.normal_scale;
		N = normalize(TBN * raw_normal);
	}

	vec4 view_pos = scene.view * vec4(in_world_pos, 1.0);
	float view_z = -view_pos.z;

	vec3 V = normalize(scene.camera_position.xyz - in_world_pos);

	vec3 F0 = vec3(0.04);
	F0 = mix(F0, albedo.rgb, metallic);

	vec3 Lo = vec3(0.0);

	// 1. Evaluate Directional Lights
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
		float is_trans = step(albedo.a, 0.999);
		vec3 kD_eff = mix(kD, kD * albedo.a, is_trans);

		float NdotL = max(dot(N, L), 0.0);

		float shadow_factor = 1.0;
		if (scene.dir_lights[i].direction.w > 0.5 && i == 0u) {
			shadow_factor = evaluate_csm_shadow(scene, in_world_pos, N, view_z);
		}

		Lo += (kD_eff * albedo.rgb / PI + specular) * radiance * NdotL * shadow_factor;
	}

	// 2. Evaluate Clustered Point & Spot Lights
	if (scene.cluster_grid_address != 0) {
		ClusterGridBuffer cluster_grid = ClusterGridBuffer(scene.cluster_grid_address);

		// Calculate tile X and Y
		uint tile_x = uint(gl_FragCoord.x * float(scene.cluster_dims.x) / scene.cluster_screen_size.x);
		uint tile_y = uint(gl_FragCoord.y * float(scene.cluster_dims.y) / scene.cluster_screen_size.y);
		tile_x = min(tile_x, scene.cluster_dims.x - 1u);
		tile_y = min(tile_y, scene.cluster_dims.y - 1u);

		// Calculate slice Z
		uint slice_z = 0u;
		if (view_z > scene.z_near && scene.z_far > scene.z_near) {
			float log_ratio = float(scene.cluster_dims.z) / log(scene.z_far / scene.z_near);
			slice_z = uint(max(0.0, floor(log(view_z / scene.z_near) * log_ratio)));
			slice_z = min(slice_z, scene.cluster_dims.z - 1u);
		}

		uint cluster_idx = tile_x + tile_y * scene.cluster_dims.x + slice_z * (scene.cluster_dims.x * scene.cluster_dims.y);
		ClusterRecord record = cluster_grid.records[cluster_idx];

		// A. Clustered Point Lights
		if (scene.point_lights_address != 0 && scene.point_light_index_list_address != 0 && record.point_count > 0) {
			PointLightBuffer pt_buf = PointLightBuffer(scene.point_lights_address);
			LightIndexBuffer pt_idx_list = LightIndexBuffer(scene.point_light_index_list_address);
			uint pt_count = min(record.point_count, scene.cluster_dims.w);

			for (uint i = 0u; i < pt_count; ++i) {
				uint light_idx = pt_idx_list.indices[record.point_offset + i];
				if (light_idx >= scene.point_light_count) continue;

				PointLight pl = pt_buf.lights[light_idx];
				vec3 light_vec = pl.position.xyz - in_world_pos;
				float distance = length(light_vec);
				float radius = pl.position.w;

				if (distance > radius) continue;

				vec3 L = normalize(light_vec);
				vec3 H = normalize(V + L);

				float atten = clamp(1.0 - pow(distance / radius, 4.0), 0.0, 1.0);
				atten = (atten * atten) / (distance * distance + 1.0);

				vec3 radiance = pl.color.rgb * pl.color.w * atten;

				float NDF = distribution_ggx(N, H, roughness);
				float G = geometry_smith(N, V, L, roughness);
				vec3 F = fresnel_schlick(max(dot(H, V), 0.0), F0);

				vec3 numerator = NDF * G * F;
				float denominator = 4.0 * max(dot(N, V), 0.0) * max(dot(N, L), 0.0) + 0.0001;
				vec3 specular = numerator / denominator;

				vec3 kS = F;
				vec3 kD = (vec3(1.0) - kS) * (1.0 - metallic);
				float is_trans = step(albedo.a, 0.999);
				vec3 kD_eff = mix(kD, kD * albedo.a, is_trans);

				float NdotL = max(dot(N, L), 0.0);
				Lo += (kD_eff * albedo.rgb / PI + specular) * radiance * NdotL;
			}
		}

		// B. Clustered Spot Lights
		if (scene.spot_lights_address != 0 && scene.spot_light_index_list_address != 0 && record.spot_count > 0) {
			SpotLightBuffer spot_buf = SpotLightBuffer(scene.spot_lights_address);
			LightIndexBuffer spot_idx_list = LightIndexBuffer(scene.spot_light_index_list_address);
			uint spot_count = min(record.spot_count, 64u);

			for (uint i = 0u; i < spot_count; ++i) {
				uint light_idx = spot_idx_list.indices[record.spot_offset + i];
				if (light_idx >= scene.spot_light_count) continue;

				SpotLight spot = spot_buf.lights[light_idx];
				vec3 light_vec = spot.position_radius.xyz - in_world_pos;
				float distance = length(light_vec);
				float radius = spot.position_radius.w;

				if (distance > radius) continue;

				vec3 L = normalize(light_vec);

				// Angular cutoff (light shines in spot.direction_cos_inner.xyz)
				float cos_theta = dot(-L, spot.direction_cos_inner.xyz);
				float cos_inner = spot.direction_cos_inner.w;
				float cos_outer = spot.params.x;

				if (cos_theta < cos_outer) continue;

				float epsilon = max(0.0001, cos_inner - cos_outer);
				float spot_atten = clamp((cos_theta - cos_outer) / epsilon, 0.0, 1.0);
				spot_atten = spot_atten * spot_atten;

				// Distance attenuation
				float dist_atten = clamp(1.0 - pow(distance / radius, 4.0), 0.0, 1.0);
				dist_atten = (dist_atten * dist_atten) / (distance * distance + 1.0);

				vec3 radiance = spot.color_intensity.rgb * spot.color_intensity.w * dist_atten * spot_atten;

				vec3 H = normalize(V + L);
				float NDF = distribution_ggx(N, H, roughness);
				float G = geometry_smith(N, V, L, roughness);
				vec3 F = fresnel_schlick(max(dot(H, V), 0.0), F0);

				vec3 numerator = NDF * G * F;
				float denominator = 4.0 * max(dot(N, V), 0.0) * max(dot(N, L), 0.0) + 0.0001;
				vec3 specular = numerator / denominator;

				vec3 kS = F;
				vec3 kD = (vec3(1.0) - kS) * (1.0 - metallic);
				float is_trans = step(albedo.a, 0.999);
				vec3 kD_eff = mix(kD, kD * albedo.a, is_trans);

				float NdotL = max(dot(N, L), 0.0);
				Lo += (kD_eff * albedo.rgb / PI + specular) * radiance * NdotL;
			}
		}
	}

	vec3 emissive = mat.emissive.rgb * mat.emissive.w;
	if (mat.emissive_texture_id != 0xFFFFFFFFu) {
		vec3 tex_emissive = texture(u_textures[nonuniformEXT(mat.emissive_texture_id)], in_uv).rgb;
		emissive *= tex_emissive;
	}

	float is_trans = step(albedo.a, 0.999);
	vec3 ambient = scene.ambient_color.rgb * albedo.rgb * occlusion * mix(1.0, albedo.a, is_trans);
	vec3 color = ambient + Lo + emissive;

	// Tonemapping and gamma correction for sRGB display
	color = tonemap_aces(color);
	color = linear_to_srgb(color);

	float NdotV = max(dot(N, V), 0.0);
	float fresnel_factor = pow(1.0 - NdotV, 5.0);
	float final_alpha = mix(albedo.a, clamp(albedo.a + (1.0 - albedo.a) * fresnel_factor * (1.0 - roughness), 0.0, 1.0), is_trans);

	out_color = vec4(color, final_alpha);
}
