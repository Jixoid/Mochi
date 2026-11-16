#version 450
#extension GL_EXT_nonuniform_qualifier : enable

layout(location = 0) in vec2 in_uv;
layout(location = 0) out vec4 out_color;

layout(set = 0, binding = 0) uniform sampler2D u_textures[];

layout(push_constant) uniform TaaPushConstants {
	vec4 rt_metrics; // (1.0/w, 1.0/h, w, h)
	uint color_tex_id;
	uint history_tex_id;
	uint velocity_tex_id;
	uint depth_tex_id;
	uint has_history;
	float blend_factor; // default 0.08
	float _pad;
} pc;

// Convert RGB to YCoCg space
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

// Intersect segment from center to history with AABB in YCoCg space (Karis / UE4 standard)
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

// 9-tap Catmull-Rom bicubic history filter (MJP standard)
// Samples a 4x4 texel footprint using 9 hardware bilinear fetches with exact center alignment
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

// Velocity dilation: find the closest depth in 3x3 to pick foreground velocity on edges
vec2 get_dilated_velocity(vec2 uv, vec2 texel) {
	float min_depth = 1.0;
	vec2 best_uv = uv;

	for (int y = -1; y <= 1; ++y) {
		for (int x = -1; x <= 1; ++x) {
			vec2 sample_uv = uv + vec2(float(x), float(y)) * texel;
			float d = texture(u_textures[nonuniformEXT(pc.depth_tex_id)], sample_uv).r;
			if (d < min_depth) {
				min_depth = d;
				best_uv = sample_uv;
			}
		}
	}

	return texture(u_textures[nonuniformEXT(pc.velocity_tex_id)], best_uv).xy;
}

void main() {
	vec2 texel = pc.rt_metrics.xy;

	// 1. Sample current center pixel
	vec3 current_rgb = texture(u_textures[nonuniformEXT(pc.color_tex_id)], in_uv).rgb;
	vec3 current_ycocg = rgb_to_ycocg(current_rgb);

	// If no previous history exists, output current color directly
	if (pc.has_history == 0) {
		out_color = vec4(current_rgb, 1.0);
		return;
	}

	// 2. Sample 3x3 neighborhood in current frame with variance clipping (Karis standard)
	vec3 m1 = current_ycocg;
	vec3 m2 = current_ycocg * current_ycocg;
	vec3 neighbor_min = current_ycocg;
	vec3 neighbor_max = current_ycocg;

	for (int y = -1; y <= 1; ++y) {
		for (int x = -1; x <= 1; ++x) {
			if (x == 0 && y == 0) continue;
			vec2 sample_uv = in_uv + vec2(float(x), float(y)) * texel;
			vec3 c = rgb_to_ycocg(texture(u_textures[nonuniformEXT(pc.color_tex_id)], sample_uv).rgb);
			m1 += c;
			m2 += c * c;
			neighbor_min = min(neighbor_min, c);
			neighbor_max = max(neighbor_max, c);
		}
	}

	vec3 mu = m1 / 9.0;
	vec3 sigma = sqrt(max(vec3(0.0), m2 / 9.0 - mu * mu));
	const float gamma = 1.25;
	vec3 box_min = max(neighbor_min, mu - gamma * sigma);
	vec3 box_max = min(neighbor_max, mu + gamma * sigma);

	// 3. Dilated velocity & Reprojection
	vec2 velocity = get_dilated_velocity(in_uv, texel);
	vec2 prev_uv = in_uv - velocity;

	// Check if reprojected UV is within screen bounds
	if (prev_uv.x < 0.0 || prev_uv.x > 1.0 || prev_uv.y < 0.0 || prev_uv.y > 1.0) {
		out_color = vec4(current_rgb, 1.0);
		return;
	}

	// 4. Sample history with 9-tap Catmull-Rom filter
	vec4 history_sample = sample_history_catmull_rom(pc.history_tex_id, prev_uv, texel);
	vec3 history_ycocg = rgb_to_ycocg(history_sample.rgb);

	// 5. AABB clipping against the 3x3 neighborhood (eliminates ghosting while preserving subpixel edge history)
	vec3 clipped_history_ycocg = clip_aabb(box_min, box_max, history_ycocg);

	// 6. Temporal blend in YCoCg space
	float alpha = clamp(pc.blend_factor, 0.01, 0.5);

	// Increase alpha during fast motion to avoid temporal lag
	float vel_mag = max(0.0, length(velocity * pc.rt_metrics.zw) - 1.0);
	alpha = mix(alpha, 0.35, clamp(vel_mag * 0.05, 0.0, 0.8));

	vec3 blended_ycocg = mix(clipped_history_ycocg, current_ycocg, alpha);

	// 7. Convert back to RGB and output
	vec3 resolved_rgb = ycocg_to_rgb(blended_ycocg);
	out_color = vec4(resolved_rgb, 1.0);
}
