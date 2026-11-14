#version 450
#extension GL_EXT_nonuniform_qualifier : enable

layout(location = 0) in vec2 in_uv;
layout(location = 0) out vec4 out_color;

layout(set = 0, binding = 0) uniform sampler2D u_textures[];

layout(push_constant) uniform Fsr3UpscalePushConstants {
	vec4 render_metrics;   // (1.0/render_w, 1.0/render_h, render_w, render_h)
	vec4 display_metrics;  // (1.0/display_w, 1.0/display_h, display_w, display_h)
	vec4 jitter;           // (jitter_ndc_x, jitter_ndc_y, jitter_px_x, jitter_px_y)
	uint color_tex_id;
	uint velocity_tex_id;
	uint depth_tex_id;
	uint history_tex_id;
	uint has_history;
	float blend_factor;
	float _pad0;
	float _pad1;
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

// Intersect segment from center to history with AABB in YCoCg space (Karis / UE4 / FSR standard)
vec3 clip_aabb(vec3 aabb_min, vec3 aabb_max, vec3 history_sample) {
	vec3 center = 0.5 * (aabb_max + aabb_min);
	vec3 extents = 0.5 * (aabb_max - aabb_min);
	vec3 offset = history_sample - center;
	vec3 abs_offset = abs(offset);
	vec3 safe_extents = max(extents, vec3(1e-5));
	vec3 units = abs_offset / safe_extents;
	float max_unit = max(units.x, max(units.y, units.z));

	if (max_unit > 1.0) {
		return center + offset / max_unit;
	} else {
		return history_sample;
	}
}

// 9-tap Catmull-Rom bicubic history filter (MJP / Karis standard)
// Samples a 4x4 display-resolution footprint using 9 bilinear fetches with exact center alignment
vec4 sample_history_catmull_rom(uint tex_id, vec2 uv, vec2 texel) {
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

// 5-tap Cross Dilated Velocity: finds foreground velocity using closest depth in cross neighborhood
vec2 get_dilated_velocity_5tap(vec2 uv, vec2 render_texel) {
	float min_depth = texture(u_textures[nonuniformEXT(pc.depth_tex_id)], uv).r;
	vec2 best_uv = uv;

	vec2 offsets[4] = vec2[](
		vec2(0.0, -render_texel.y),
		vec2(-render_texel.x, 0.0),
		vec2(render_texel.x, 0.0),
		vec2(0.0, render_texel.y)
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
	vec2 render_texel = pc.render_metrics.xy;
	vec2 display_texel = pc.display_metrics.xy;

	// 1. Sample current frame at exact in_uv (no inverted jitter, matches proven TAA reconstruction)
	vec3 current_rgb = texture(u_textures[nonuniformEXT(pc.color_tex_id)], in_uv).rgb;
	vec3 current_ycocg = rgb_to_ycocg(current_rgb);

	// Early out if no valid history exists yet
	if (pc.has_history == 0) {
		out_color = vec4(current_rgb, 1.0);
		return;
	}

	// 2. Sample 3x3 neighborhood around in_uv in render resolution for Karis AABB clipping
	vec3 box_min = current_ycocg;
	vec3 box_max = current_ycocg;

	for (int y = -1; y <= 1; ++y) {
		for (int x = -1; x <= 1; ++x) {
			if (x == 0 && y == 0) continue;
			vec2 tap_uv = in_uv + vec2(float(x), float(y)) * render_texel;
			vec3 c = rgb_to_ycocg(texture(u_textures[nonuniformEXT(pc.color_tex_id)], tap_uv).rgb);
			box_min = min(box_min, c);
			box_max = max(box_max, c);
		}
	}

	// Allow 15% subpixel expansion to preserve sharp high-resolution details in history without blurring
	vec3 box_center = 0.5 * (box_max + box_min);
	vec3 box_extents = 0.5 * (box_max - box_min) * 1.15;
	vec3 clip_min = box_center - box_extents;
	vec3 clip_max = box_center + box_extents;

	// 3. Fast 5-tap Dilated Velocity & Reprojection
	vec2 velocity = get_dilated_velocity_5tap(in_uv, render_texel);
	vec2 prev_uv = in_uv - velocity;

	// Check if reprojected UV is within screen bounds
	if (prev_uv.x < 0.0 || prev_uv.x > 1.0 || prev_uv.y < 0.0 || prev_uv.y > 1.0) {
		out_color = vec4(current_rgb, 1.0);
		return;
	}

	// 4. Sample Full-Resolution History with 9-tap Catmull-Rom Bicubic Filter
	vec4 history_sample = sample_history_catmull_rom(pc.history_tex_id, prev_uv, display_texel);
	vec3 history_ycocg = rgb_to_ycocg(history_sample.rgb);

	// 5. Karis AABB Clipping (eliminates ghosting while maintaining subpixel sharpness)
	vec3 clipped_history_ycocg = clip_aabb(clip_min, clip_max, history_ycocg);

	// 6. Motion-Adaptive Temporal Accumulation
	float vel_mag = max(0.0, length(velocity * pc.display_metrics.zw) - 1.0);
	float motion_factor = clamp(vel_mag * 0.05, 0.0, 0.85);

	float base_alpha = clamp(pc.blend_factor, 0.03, 0.5);
	float alpha = mix(base_alpha, 0.35, motion_factor);

	vec3 blended_ycocg = mix(clipped_history_ycocg, current_ycocg, alpha);

	// 7. Convert back to RGB and output to display-resolution history buffer
	vec3 resolved_rgb = ycocg_to_rgb(blended_ycocg);
	out_color = vec4(resolved_rgb, 1.0);
}
