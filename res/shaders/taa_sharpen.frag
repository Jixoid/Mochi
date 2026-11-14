#version 450
#extension GL_EXT_nonuniform_qualifier : enable

layout(location = 0) in vec2 in_uv;
layout(location = 0) out vec4 out_color;

layout(set = 0, binding = 0) uniform sampler2D u_textures[];

layout(push_constant) uniform TaaSharpenPushConstants {
	vec4 rt_metrics; // (1.0/w, 1.0/h, w, h)
	uint texture_id;
	float sharpness; // 0.0 to 1.0 (recommended: 0.2 to 0.35)
	float _pad0;
	float _pad1;
} pc;

void main() {
	vec2 texel = pc.rt_metrics.xy;

	vec3 e = texture(u_textures[nonuniformEXT(pc.texture_id)], in_uv).rgb;

	// If sharpness is 0, pass-through directly
	if (pc.sharpness <= 0.001) {
		out_color = vec4(e, 1.0);
		return;
	}

	vec3 b = texture(u_textures[nonuniformEXT(pc.texture_id)], in_uv + vec2(0.0, -texel.y)).rgb;
	vec3 d = texture(u_textures[nonuniformEXT(pc.texture_id)], in_uv + vec2(-texel.x, 0.0)).rgb;
	vec3 f = texture(u_textures[nonuniformEXT(pc.texture_id)], in_uv + vec2(texel.x, 0.0)).rgb;
	vec3 h = texture(u_textures[nonuniformEXT(pc.texture_id)], in_uv + vec2(0.0, texel.y)).rgb;

	// Contrast-Adaptive Sharpening (RCAS):
	// Min/max contrast in cross neighborhood
	vec3 min_rgb = min(min(b, d), min(f, h));
	min_rgb = min(min_rgb, e);

	vec3 max_rgb = max(max(b, d), max(f, h));
	max_rgb = max(max_rgb, e);

	// Ringing suppression
	vec3 contrast = min(min_rgb, vec3(2.0) - max_rgb);
	vec3 amp = clamp(contrast / (max_rgb + 1e-5), 0.0, 1.0);
	vec3 w = amp * (-pc.sharpness * 0.25);

	vec3 sharpened = (b * w + d * w + f * w + h * w + e) / (vec3(1.0) + vec3(4.0) * w);
	sharpened = clamp(sharpened, min_rgb, max_rgb);

	out_color = vec4(sharpened, 1.0);
}
