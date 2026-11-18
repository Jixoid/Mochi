/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use mochi_math::{Mat4x4, Vec3, Vec4};

use crate::frame::MAX_DIRECTIONAL_LIGHTS;
use mochi_rhi::{
	self as rhi, Format, ImageUsage, PipelineMeta, SamplerAddressMode, SamplerFilter, Shader,
	ShaderStage,
};
use mochi_scene::{CameraComponent, TransformComponent};


pub const SHADOW_MAP_SIZE: u32 = 2048;
pub const SPOT_SHADOW_MAP_SIZE: u32 = 1024;
pub const CASCADE_COUNT: usize = 4;
pub const CASCADE_SPLITS: [f32; CASCADE_COUNT] = [2.5, 8.0, 24.0, 80.0];
pub const MAX_SHADOW_CASTING_SPOT_LIGHTS: usize = 4;

pub const SHADOW_VERT_SPV: &[u8] = include_bytes!("../../../res/shaders/shadow.vert.spv");

#[repr(C, align(16))]
#[derive(Copy, Clone, Debug)]
pub struct ShadowPushConstants {
	pub mvp: [f32; 16],
	pub vertex_addr: u64,
	pub joint_addr: u64,
}

pub struct CascadeData {
	pub image: rhi::Image2,
	pub view: rhi::ImageView2,
	pub descriptor_id: u32,
	pub view_proj: [f32; 16],
}

pub struct CascadeSet {
	pub cascades: [CascadeData; CASCADE_COUNT],
}

pub struct SpotShadowData {
	pub image: rhi::Image2,
	pub view: rhi::ImageView2,
	pub descriptor_id: u32,
	pub view_proj: [f32; 16],
}

pub struct ShadowManager {
	pub cascade_sets: [[Option<CascadeSet>; MAX_DIRECTIONAL_LIGHTS]; 2],
	pub spot_shadow_sets: [[Option<SpotShadowData>; MAX_SHADOW_CASTING_SPOT_LIGHTS]; 2],
	pub active_spot_counts: [usize; 2],
	pub sampler: rhi::Sampler2,
	pub pipeline: rhi::Pipeline,
	pub meta: rhi::PipelineMeta,
	pub split_depths: [f32; CASCADE_COUNT],
}

impl ShadowManager {
	pub fn new(device: &rhi::Device, _alloc: &rhi::Allocator) -> Result<Self, String> {
		let sampler = device
			.create_sampler(SamplerFilter::Linear, SamplerAddressMode::ClampToBorder)
			.map_err(|e| format!("Failed to create shadow comparison sampler: {e}"))?;

		let shadow_vert = Shader::new(device, ShaderStage::Vertex, SHADOW_VERT_SPV, "main");

		let meta = PipelineMeta::new_raw(
			ShaderStage::Vertex,
			std::mem::size_of::<ShadowPushConstants>() as u32,
		);

		let pipeline = rhi::GraphicsPipelineBuilder::new(device, &meta)
			.shader(&shadow_vert)
			.depth_format(Format::Depth32f)
			.build();

		Ok(Self {
			cascade_sets: [[None, None, None, None], [None, None, None, None]],
			spot_shadow_sets: [[None, None, None, None], [None, None, None, None]],
			active_spot_counts: [0, 0],
			sampler,
			pipeline,
			meta,
			split_depths: CASCADE_SPLITS,
		})
	}

	pub fn ensure_cascade_set(
		&mut self,
		device: &rhi::Device,
		alloc: &rhi::Allocator,
		frame_idx: usize,
		light_idx: usize,
	) -> Result<(), String> {
		if light_idx >= MAX_DIRECTIONAL_LIGHTS {
			return Err(format!(
				"Light index {light_idx} exceeds MAX_DIRECTIONAL_LIGHTS"
			));
		}

		if self.cascade_sets[frame_idx][light_idx].is_some() {
			return Ok(());
		}

		let mut cascade_list = Vec::with_capacity(CASCADE_COUNT);

		for _i in 0..CASCADE_COUNT {
			let image = rhi::Image2::builder(SHADOW_MAP_SIZE, SHADOW_MAP_SIZE, Format::Depth32f)
				.usage(ImageUsage::DepthStencilAttachment | ImageUsage::Sampled)
				.build(alloc);

			let view = device
				.create_image_view(&image, Format::Depth32f)
				.map_err(|e| format!("Failed to create cascade depth image view: {e}"))?;

			let descriptor_id = device.write_texture_descriptor(&view, &self.sampler);

			cascade_list.push(CascadeData {
				image,
				view,
				descriptor_id,
				view_proj: Mat4x4::IDENTITY.to_array(),
			});
		}

		let cascades: [CascadeData; CASCADE_COUNT] = cascade_list
			.try_into()
			.map_err(|_| "Failed to collect 4 cascades".to_string())?;

		self.cascade_sets[frame_idx][light_idx] = Some(CascadeSet { cascades });
		Ok(())
	}

	pub fn update_cascades(
		&mut self,
		frame_idx: usize,
		light_idx: usize,
		camera: &CameraComponent,
		camera_transform: &TransformComponent,
		aspect: f32,
		sun_dir: Vec3<f32>,
	) {
		if light_idx >= MAX_DIRECTIONAL_LIGHTS {
			return;
		}

		let set = match &mut self.cascade_sets[frame_idx][light_idx] {
			Some(s) => s,
			None => return,
		};

		let dir = if sun_dir.dot(&sun_dir) < 1e-4 {
			Vec3::new(0.5, -1.0, 0.3).normalize()
		} else {
			sun_dir.normalize()
		};

		let up = if dir.y.abs() > 0.99 {
			Vec3::new(0.0, 0.0, 1.0)
		} else {
			Vec3::new(0.0, 1.0, 0.0)
		};

		let fov_rad = camera.fov_deg.to_radians();
		let tan_half_fov = (fov_rad * 0.5).tan();

		let cam_pos = camera_transform.translation;
		let cam_rot = camera_transform.rotation;
		let cam_forward = cam_rot * Vec3::new(0.0, 0.0, -1.0);

		for c in 0..CASCADE_COUNT {
			let z_near = if c == 0 {
				camera.near
			} else {
				CASCADE_SPLITS[c - 1]
			};
			let z_far = CASCADE_SPLITS[c].min(camera.far);

			// Frustum slice dimensions at near and far split depths
			let near_h = z_near * tan_half_fov;
			let near_w = near_h * aspect;
			let r_near_sq = near_w * near_w + near_h * near_h;

			let far_h = z_far * tan_half_fov;
			let far_w = far_h * aspect;
			let r_far_sq = far_w * far_w + far_h * far_h;

			// Optimal circumscribed sphere center along view axis:
			// d = (z_far + z_near)/2 + (r_far^2 - r_near^2) / (2 * (z_far - z_near))
			let d = 0.5 * (z_far + z_near) + (r_far_sq - r_near_sq) / (2.0 * (z_far - z_near));
			let radius = (r_far_sq + (z_far - d) * (z_far - d)).sqrt() * 1.02;

			// Circumcenter in world space
			let center = cam_pos + cam_forward * d;

			const FAR_OFFSET: f32 = 100.0;
			let eye = center - dir * (radius + FAR_OFFSET);
			let light_view = Mat4x4::look_at(eye, center, up);

			let near_z = 0.0;
			let far_z = 2.0 * radius + FAR_OFFSET + 50.0;
			let mut light_proj = Mat4x4::orthographic(-radius, radius, -radius, radius, near_z, far_z);

			// Michal Valient / Stable CSM Stabilization:
			let shadow_matrix = light_proj * light_view;
			let shadow_origin = shadow_matrix * Vec4::new_with(0.0, 0.0, 0.0, 1.0);

			let half_size = SHADOW_MAP_SIZE as f32 * 0.5;
			let texel_origin_x = shadow_origin.x * half_size;
			let texel_origin_y = shadow_origin.y * half_size;

			let round_offset_x =
				(texel_origin_x.round() - texel_origin_x) * (2.0 / SHADOW_MAP_SIZE as f32);
			let round_offset_y =
				(texel_origin_y.round() - texel_origin_y) * (2.0 / SHADOW_MAP_SIZE as f32);

			light_proj.rows[0].w += round_offset_x;
			light_proj.rows[1].w += round_offset_y;

			let view_proj = light_proj * light_view;
			set.cascades[c].view_proj = view_proj.to_array();
		}
	}

	pub fn cascade_view_proj(
		&self,
		frame_idx: usize,
		light_idx: usize,
	) -> [[f32; 16]; CASCADE_COUNT] {
		if light_idx < MAX_DIRECTIONAL_LIGHTS
			&& let Some(set) = &self.cascade_sets[frame_idx][light_idx]
		{
			return [
				set.cascades[0].view_proj,
				set.cascades[1].view_proj,
				set.cascades[2].view_proj,
				set.cascades[3].view_proj,
			];
		}
		[Mat4x4::IDENTITY.to_array(); CASCADE_COUNT]
	}

	pub fn cascade_split_depths(&self) -> [f32; CASCADE_COUNT] {
		self.split_depths
	}

	pub fn cascade_shadow_ids(&self, frame_idx: usize, light_idx: usize) -> [u32; CASCADE_COUNT] {
		if light_idx < MAX_DIRECTIONAL_LIGHTS
			&& let Some(set) = &self.cascade_sets[frame_idx][light_idx]
		{
			return [
				set.cascades[0].descriptor_id,
				set.cascades[1].descriptor_id,
				set.cascades[2].descriptor_id,
				set.cascades[3].descriptor_id,
			];
		}
		[0; CASCADE_COUNT]
	}

	pub fn ensure_spot_shadow_set(
		&mut self,
		device: &rhi::Device,
		alloc: &rhi::Allocator,
		frame_idx: usize,
		slot: usize,
	) -> Result<(), String> {
		if slot >= MAX_SHADOW_CASTING_SPOT_LIGHTS {
			return Err(format!(
				"Spot shadow slot {slot} exceeds MAX_SHADOW_CASTING_SPOT_LIGHTS"
			));
		}

		if self.spot_shadow_sets[frame_idx][slot].is_some() {
			return Ok(());
		}

		let image = rhi::Image2::builder(SPOT_SHADOW_MAP_SIZE, SPOT_SHADOW_MAP_SIZE, Format::Depth32f)
			.usage(ImageUsage::DepthStencilAttachment | ImageUsage::Sampled)
			.build(alloc);

		let view = device
			.create_image_view(&image, Format::Depth32f)
			.map_err(|e| format!("Failed to create spot shadow depth image view: {e}"))?;

		let descriptor_id = device.write_texture_descriptor(&view, &self.sampler);

		self.spot_shadow_sets[frame_idx][slot] = Some(SpotShadowData {
			image,
			view,
			descriptor_id,
			view_proj: Mat4x4::IDENTITY.to_array(),
		});

		Ok(())
	}

	pub fn update_spot_shadow(
		&mut self,
		frame_idx: usize,
		slot: usize,
		pos: Vec3<f32>,
		dir: Vec3<f32>,
		outer_cone_angle_deg: f32,
		radius: f32,
	) {
		if slot >= MAX_SHADOW_CASTING_SPOT_LIGHTS {
			return;
		}

		let spot = match &mut self.spot_shadow_sets[frame_idx][slot] {
			Some(s) => s,
			None => return,
		};

		let dir_norm = if dir.dot(&dir) < 1e-4 {
			Vec3::new(0.0, 0.0, -1.0)
		} else {
			dir.normalize()
		};

		let up = if dir_norm.y.abs() > 0.99 {
			Vec3::new(0.0, 0.0, 1.0)
		} else {
			Vec3::new(0.0, 1.0, 0.0)
		};

		let light_view = Mat4x4::look_at(pos, pos + dir_norm, up);
		let near_z = 0.05_f32;
		let far_z = radius.max(near_z + 0.1);
		let fov = outer_cone_angle_deg.clamp(1.0, 170.0);
		let light_proj = Mat4x4::perspective(fov, 1.0, near_z, far_z);

		let view_proj = light_proj * light_view;
		spot.view_proj = view_proj.to_array();
	}

	pub fn spot_shadow_view_proj(&self, frame_idx: usize, slot: usize) -> [f32; 16] {
		if slot < MAX_SHADOW_CASTING_SPOT_LIGHTS
			&& let Some(spot) = &self.spot_shadow_sets[frame_idx][slot]
		{
			return spot.view_proj;
		}
		Mat4x4::IDENTITY.to_array()
	}

	pub fn spot_shadow_id(&self, frame_idx: usize, slot: usize) -> u32 {
		if slot < MAX_SHADOW_CASTING_SPOT_LIGHTS
			&& let Some(spot) = &self.spot_shadow_sets[frame_idx][slot]
		{
			return spot.descriptor_id;
		}
		0
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use mochi_math::Vec4;

	#[test]
	fn test_shadow_projection_stability() {
		let dir = Vec3::new(0.5_f32, -1.0, 0.3).normalize();
		let up = Vec3::new(0.0, 1.0, 0.0);
		let radius = 6.0_f32;
		let shadow_map_size = 2048.0_f32;

		const FAR_OFFSET: f32 = 100.0;
		let near_z = 0.0;
		let far_z = 2.0 * radius + FAR_OFFSET + 50.0;

		let valient_vp = |center: Vec3<f32>| -> Mat4x4<f32> {
			let eye = center - dir * (radius + FAR_OFFSET);
			let light_view = Mat4x4::look_at(eye, center, up);
			let mut light_proj = Mat4x4::orthographic(-radius, radius, -radius, radius, near_z, far_z);

			let shadow_matrix = light_proj * light_view;
			let shadow_origin = shadow_matrix * Vec4::new_with(0.0, 0.0, 0.0, 1.0);

			let half_size = shadow_map_size * 0.5;
			let texel_origin_x = shadow_origin.x * half_size;
			let texel_origin_y = shadow_origin.y * half_size;

			let round_offset_x = (texel_origin_x.round() - texel_origin_x) * (2.0 / shadow_map_size);
			let round_offset_y = (texel_origin_y.round() - texel_origin_y) * (2.0 / shadow_map_size);

			light_proj.rows[0].w += round_offset_x;
			light_proj.rows[1].w += round_offset_y;

			light_proj * light_view
		};

		let p_world = Vec4::new_with(0.0, 0.0, 0.0, 1.0);
		let base_center = Vec3::new(0.0, 0.4, 1.0);

		// Verify that in all directions (+/- X, +/- Y, +/- Z), sub-texel phase error is strictly 0
		for step in -10..=10 {
			let center =
				base_center + Vec3::new(step as f32 * 0.03, step as f32 * -0.02, step as f32 * 0.05);
			let vp = valient_vp(center);
			let clip = vp * p_world;
			let uv = (clip.x * 0.5 + 0.5, clip.y * 0.5 + 0.5);
			let tex = (uv.0 * shadow_map_size, uv.1 * shadow_map_size);
			let frac_x = (tex.0 - tex.0.round()).abs();
			let frac_y = (tex.1 - tex.1.round()).abs();
			assert!(
				frac_x < 1e-4,
				"Subtexel phase error on X for step {}: {}",
				step,
				frac_x
			);
			assert!(
				frac_y < 1e-4,
				"Subtexel phase error on Y for step {}: {}",
				step,
				frac_y
			);
		}
	}

	#[test]
	fn test_spot_shadow_projection_bounds() {
		let pos = Vec3::new(0.0_f32, 5.0, 0.0);
		let dir = Vec3::new(0.0_f32, -1.0, 0.0); // shines straight down
		let up = Vec3::new(0.0_f32, 0.0, 1.0);
		let outer_cone_deg = 60.0_f32;
		let radius = 10.0_f32;
		let near_z = 0.05_f32;

		let view = Mat4x4::look_at(pos, pos + dir, up);
		let proj = Mat4x4::perspective(outer_cone_deg, 1.0, near_z, radius);
		let vp = proj * view;

		// 1. Center point at near_z: (0, 5 - 0.05, 0) -> clip z/w should be near 0
		let p_near = Vec4::new_with(0.0, 5.0 - near_z, 0.0, 1.0);
		let clip_near = vp * p_near;
		let z_near_ndc = clip_near.z / clip_near.w;
		assert!(
			(z_near_ndc - 0.0).abs() < 1e-3,
			"Near depth should be ~0.0, got {z_near_ndc}"
		);

		// 2. Center point at far_z (radius): (0, 5 - 10, 0) -> clip z/w should be near 1.0
		let p_far = Vec4::new_with(0.0, 5.0 - radius, 0.0, 1.0);
		let clip_far = vp * p_far;
		let z_far_ndc = clip_far.z / clip_far.w;
		assert!(
			(z_far_ndc - 1.0).abs() < 1e-3,
			"Far depth should be ~1.0, got {z_far_ndc}"
		);

		// 3. Center point in middle: UV should be center (0.5, 0.5)
		let p_mid = Vec4::new_with(0.0, 2.0, 0.0, 1.0);
		let clip_mid = vp * p_mid;
		let uv_x = (clip_mid.x / clip_mid.w) * 0.5 + 0.5;
		let uv_y = (clip_mid.y / clip_mid.w) * 0.5 + 0.5;
		assert!(
			(uv_x - 0.5).abs() < 1e-4,
			"Center point UV X should be 0.5, got {uv_x}"
		);
		assert!(
			(uv_y - 0.5).abs() < 1e-4,
			"Center point UV Y should be 0.5, got {uv_y}"
		);

		// 4. Point behind light: w should be negative or <= 0
		let p_behind = Vec4::new_with(0.0, 10.0, 0.0, 1.0);
		let clip_behind = vp * p_behind;
		assert!(
			clip_behind.w < 0.0,
			"Point behind light must have w < 0, got {}",
			clip_behind.w
		);
	}
}
