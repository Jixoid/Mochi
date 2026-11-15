#version 450
#extension GL_EXT_nonuniform_qualifier : enable

layout(location = 0) in vec2 in_uv;
layout(location = 0) out vec4 out_color;

layout(set = 0, binding = 0) uniform sampler2D u_textures[];

layout(push_constant) uniform Fsr3FrameGenPushConstants {
	vec4 display_metrics;          // (1.0/w, 1.0/h, w, h)
	uint prev_color_tex_id;
	uint curr_color_tex_id;
	uint velocity_tex_id;
	uint depth_tex_id;
	float interpolation_factor;    // 0.5 (mid-frame)
	float disocclusion_threshold;  // 0.08
	uint has_history;              // 1 if prev frame is valid, 0 otherwise
	float _pad0;
} pc;

// Convert RGB to YCoCg color space
vec3 rgb_to_ycocg(vec3 rgb) {
	float y  =  0.25 * rgb.r + 0.50 * rgb.g + 0.25 * rgb.b;
	float co =  0.50 * rgb.r               - 0.50 * rgb.b;
	float cg = -0.25 * rgb.r + 0.50 * rgb.g - 0.25 * rgb.b;
	return vec3(y, co, cg);
}

// Convert YCoCg space back to RGB
vec3 ycocg_to_rgb(vec3 ycocg) {
	float y  = ycocg.x;
	float co = ycocg.y;
	float cg = ycocg.z;
	float r = y + co - cg;
	float g = y + cg;
	float b = y - co - cg;
	return max(vec3(0.0), vec3(r, g, b));
}

// Intersect segment with AABB in YCoCg space (Karis / UE / FSR standard)
vec3 clip_aabb(vec3 aabb_min, vec3 aabb_max, vec3 sample_val) {
	vec3 center = 0.5 * (aabb_max + aabb_min);
	vec3 extents = 0.5 * (aabb_max - aabb_min);
	vec3 offset = sample_val - center;
	vec3 abs_offset = abs(offset);
	vec3 safe_extents = max(extents, vec3(1e-5));
	vec3 units = abs_offset / safe_extents;
	float max_unit = max(units.x, max(units.y, units.z));

	if (max_unit > 1.0) {
		return center + offset / max_unit;
	} else {
		return sample_val;
	}
}

// 9-tap Catmull-Rom bicubic history filter (preserves fine subpixel geometry)
vec4 sample_catmull_rom(uint tex_id, vec2 uv, vec2 texel) {
	vec2 samplePos = uv / texel;
	vec2 texPos1 = floor(samplePos - 0.5) + 0.5;
	vec2 f = samplePos - texPos1;

	vec2 w0 = f * (-0.5 + f * (1.0 - 0.5 * f));
	vec2 w1 = 1.0 + f * f * (-2.5 + 1.5 * f);
	vec2 w2 = f * (0.5 + f * (2.0 - 1.5 * f));
	vec2 w3 = f * f * (-0.5 + 0.5 * f);

	vec2 w12 = w1 + w2;
	vec2 offset12 = w2 / (w12 + vec2(1e-5));

	vec2 texPos0 = (texPos1 - 1.0) * texel;
	vec2 texPos3 = (texPos1 + 2.0) * texel;
	vec2 texPos12 = (texPos1 + offset12) * texel;

	vec4 result = vec4(0.0);
	result += texture(u_textures[nonuniformEXT(tex_id)], vec2(texPos0.x, texPos0.y)) * (w0.x * w0.y);
	result += texture(u_textures[nonuniformEXT(tex_id)], vec2(texPos12.x, texPos0.y)) * (w12.x * w0.y);
	result += texture(u_textures[nonuniformEXT(tex_id)], vec2(texPos3.x, texPos0.y)) * (w3.x * w0.y);

	result += texture(u_textures[nonuniformEXT(tex_id)], vec2(texPos0.x, texPos12.y)) * (w0.x * w12.y);
	result += texture(u_textures[nonuniformEXT(tex_id)], vec2(texPos12.x, texPos12.y)) * (w12.x * w12.y);
	result += texture(u_textures[nonuniformEXT(tex_id)], vec2(texPos3.x, texPos12.y)) * (w3.x * w12.y);

	result += texture(u_textures[nonuniformEXT(tex_id)], vec2(texPos0.x, texPos3.y)) * (w0.x * w3.y);
	result += texture(u_textures[nonuniformEXT(tex_id)], vec2(texPos12.x, texPos3.y)) * (w12.x * w3.y);
	result += texture(u_textures[nonuniformEXT(tex_id)], vec2(texPos3.x, texPos3.y)) * (w3.x * w3.y);

	return max(vec4(0.0), result);
}

// 5-tap dilated velocity guided by closest depth (foreground dominance)
vec2 get_dilated_velocity_5tap(vec2 uv, vec2 texel) {
	if (pc.velocity_tex_id == 0 || pc.depth_tex_id == 0) {
		return vec2(0.0);
	}

	float min_depth = texture(u_textures[nonuniformEXT(pc.depth_tex_id)], uv).r;
	vec2 best_uv = uv;

	vec2 offsets[4] = vec2[](
		vec2(0.0, -texel.y),
		vec2(-texel.x, 0.0),
		vec2(texel.x, 0.0),
		vec2(0.0, texel.y)
	);

	for (int i = 0; i < 4; ++i) {
		vec2 sample_uv = uv + offsets[i];
		float d = texture(u_textures[nonuniformEXT(pc.depth_tex_id)], sample_uv).r;
		if (d < min_depth) {
			min_depth = d;
			best_uv = sample_uv;
		}
	}

	return texture(u_textures[nonuniformEXT(pc.velocity_tex_id)], best_uv).xy;
}

void main() {
	vec2 texel = pc.display_metrics.xy;

	vec3 curr_rgb = texture(u_textures[nonuniformEXT(pc.curr_color_tex_id)], in_uv).rgb;

	// If no history exists, fall back immediately to current frame
	if (pc.has_history == 0) {
		out_color = vec4(curr_rgb, 1.0);
		return;
	}

	// 1. Dilated screen-space velocity vector
	vec2 velocity = get_dilated_velocity_5tap(in_uv, texel);

	// 2. Bidirectional mid-frame displacement vectors
	float lambda = clamp(pc.interpolation_factor, 0.01, 0.99);
	vec2 v_fwd = velocity * (1.0 - lambda);
	vec2 v_bwd = velocity * lambda;

	vec2 uv_prev = in_uv - v_fwd;
	vec2 uv_curr = in_uv + v_bwd;

	// Screen bounds verification
	bool prev_valid = uv_prev.x >= 0.0 && uv_prev.x <= 1.0 && uv_prev.y >= 0.0 && uv_prev.y <= 1.0;
	bool curr_valid = uv_curr.x >= 0.0 && uv_curr.x <= 1.0 && uv_curr.y >= 0.0 && uv_curr.y <= 1.0;

	if (!prev_valid) {
		out_color = vec4(curr_rgb, 1.0);
		return;
	}

	// 3. High-quality bicubic Catmull-Rom sampling
	vec3 sample_prev_rgb = sample_catmull_rom(pc.prev_color_tex_id, uv_prev, texel).rgb;
	vec3 sample_curr_rgb = curr_valid
		? sample_catmull_rom(pc.curr_color_tex_id, uv_curr, texel).rgb
		: curr_rgb;

	vec3 prev_ycocg = rgb_to_ycocg(sample_prev_rgb);
	vec3 curr_ycocg = rgb_to_ycocg(sample_curr_rgb);
	vec3 center_curr_ycocg = rgb_to_ycocg(curr_rgb);

	// 4. Sample 3x3 neighborhood of current frame for AABB color bounding box
	vec3 box_min = center_curr_ycocg;
	vec3 box_max = center_curr_ycocg;

	for (int y = -1; y <= 1; ++y) {
		for (int x = -1; x <= 1; ++x) {
			if (x == 0 && y == 0) continue;
			vec2 tap_uv = in_uv + vec2(float(x), float(y)) * texel;
			vec3 c = rgb_to_ycocg(texture(u_textures[nonuniformEXT(pc.curr_color_tex_id)], tap_uv).rgb);
			box_min = min(box_min, c);
			box_max = max(box_max, c);
		}
	}

	// Expand box by 15% to maintain edge transitions
	vec3 box_center = 0.5 * (box_max + box_min);
	vec3 box_extents = 0.5 * (box_max - box_min) * 1.15;
	vec3 clip_min = box_center - box_extents;
	vec3 clip_max = box_center + box_extents;

	// 5. Karis AABB clipping on previous frame sample to eliminate ghosting
	vec3 clipped_prev_ycocg = clip_aabb(clip_min, clip_max, prev_ycocg);

	// 6. Luminance & color distance disocclusion fallback
	float luma_diff = abs(clipped_prev_ycocg.x - curr_ycocg.x);
	float threshold = max(0.02, pc.disocclusion_threshold);
	float disocclusion = smoothstep(threshold, threshold * 2.5, luma_diff);

	// If disoccluded, shift weight smoothly towards current frame
	float blend_weight = mix(lambda, 1.0, disocclusion);

	// 7. Bidirectional interpolation
	vec3 blended_ycocg = mix(clipped_prev_ycocg, curr_ycocg, blend_weight);
	vec3 final_rgb = ycocg_to_rgb(blended_ycocg);

	out_color = vec4(final_rgb, 1.0);
}
