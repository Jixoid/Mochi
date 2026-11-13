/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use mochi_math::{Extent, Mat4x4, Vec3, Vec4};

use crate::rendering::frame::MAX_DIRECTIONAL_LIGHTS;
use crate::rhi::{
	self, AllocationCreate, AllocationLocation, Format, ImageTiling, ImageUsage, Pipeline,
	PipelineMeta, SamplerAddressMode, SamplerFilter, Shader, ShaderStage,
};
use crate::scene::{CameraComponent, TransformComponent};


pub const SHADOW_MAP_SIZE: u32 = 2048;
pub const CASCADE_COUNT: usize = 4;
pub const CASCADE_SPLITS: [f32; CASCADE_COUNT] = [2.5, 8.0, 24.0, 80.0];

pub const SHADOW_VERT_SPV: &[u8] = include_bytes!(concat!(
	env!("CARGO_MANIFEST_DIR"),
	"/res/shaders/shadow.vert.spv"
));

#[repr(C, align(16))]
#[derive(Copy, Clone, Debug)]
pub struct ShadowPushConstants {
	pub mvp: [f32; 16],
	pub vertex_addr: u64,
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

pub struct ShadowManager {
	pub cascade_sets: [Option<CascadeSet>; MAX_DIRECTIONAL_LIGHTS],
	pub sampler: rhi::Sampler2,
	pub pipeline: rhi::Pipeline,
	pub meta: rhi::PipelineMeta,
	pub split_depths: [f32; CASCADE_COUNT],
}

impl ShadowManager {
	pub fn new(device: &rhi::Device, _alloc: &rhi::Allocator) -> Result<Self, String> {
		let sampler = device
			.create_sampler(
				SamplerFilter::Linear,
				SamplerAddressMode::ClampToBorder,
			)
			.map_err(|e| format!("Failed to create shadow comparison sampler: {e}"))?;

		let shadow_vert = Shader::new(
			device,
			ShaderStage::Vertex,
			SHADOW_VERT_SPV,
			"main",
		);

		let meta = PipelineMeta::new_raw(
			ShaderStage::Vertex,
			std::mem::size_of::<ShadowPushConstants>() as u32,
		);

		let pipeline = Pipeline::new_graphics(
			device,
			&meta,
			&[&shadow_vert],
			rhi::PolygonMode::Fill,
			rhi::PrimitiveTopology::TriangleList,
			Format::Undefined,
			Format::Depth32f,
			false,
			true,
			rhi::MsaaSamples::Off,
		);

		Ok(Self {
			cascade_sets: [None, None, None, None],
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
		light_idx: usize,
	) -> Result<(), String> {
		if light_idx >= MAX_DIRECTIONAL_LIGHTS {
			return Err(format!("Light index {light_idx} exceeds MAX_DIRECTIONAL_LIGHTS"));
		}

		if self.cascade_sets[light_idx].is_some() {
			return Ok(());
		}

		let ext = Extent::new([SHADOW_MAP_SIZE, SHADOW_MAP_SIZE]);
		let mut cascade_list = Vec::with_capacity(CASCADE_COUNT);

		for _i in 0..CASCADE_COUNT {
			let image = rhi::Image2::new(
				alloc,
				ext,
				Format::Depth32f,
				ImageUsage::DepthStencilAttachment | ImageUsage::Sampled,
				ImageTiling::Optimal,
				AllocationCreate::Dedicated,
				AllocationLocation::PreferDevice,
			);

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

		self.cascade_sets[light_idx] = Some(CascadeSet { cascades });
		Ok(())
	}

	pub fn update_cascades(
		&mut self,
		light_idx: usize,
		camera: &CameraComponent,
		camera_transform: &TransformComponent,
		aspect: f32,
		sun_dir: Vec3<f32>,
	) {
		if light_idx >= MAX_DIRECTIONAL_LIGHTS {
			return;
		}

		let set = match &mut self.cascade_sets[light_idx] {
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

		for c in 0..CASCADE_COUNT {
			let z_near = if c == 0 {
				camera.near
			} else {
				CASCADE_SPLITS[c - 1]
			};
			let z_far = CASCADE_SPLITS[c].min(camera.far);

			// Frustum corners in camera space
			let near_h = z_near * tan_half_fov;
			let near_w = near_h * aspect;
			let far_h = z_far * tan_half_fov;
			let far_w = far_h * aspect;

			let local_corners = [
				Vec3::new(-near_w, -near_h, -z_near),
				Vec3::new(near_w, -near_h, -z_near),
				Vec3::new(-near_w, near_h, -z_near),
				Vec3::new(near_w, near_h, -z_near),
				Vec3::new(-far_w, -far_h, -z_far),
				Vec3::new(far_w, -far_h, -z_far),
				Vec3::new(-far_w, far_h, -z_far),
				Vec3::new(far_w, far_h, -z_far),
			];

			// Transform corners to world space
			let mut center = Vec3::ZERO;
			let mut world_corners = [Vec3::ZERO; 8];
			for i in 0..8 {
				let world_pt = cam_pos + cam_rot * local_corners[i];
				world_corners[i] = world_pt;
				center = center + world_pt;
			}
			center = center * (1.0 / 8.0);

			// Compute bounding sphere radius
			let mut radius: f32 = 0.0;
			for i in 0..8 {
				let d = world_corners[i].dist(&center);
				if d > radius {
					radius = d;
				}
			}

			// Add small padding to radius to avoid frustum boundary artifacts
			radius = radius * 1.02;

			const FAR_OFFSET: f32 = 100.0;
			let eye = center - dir * (radius + FAR_OFFSET);
			let light_view = Mat4x4::look_at(eye, center, up);

			// Transform sphere center into light view space
			let center_view = light_view * Vec4::new_with(center.x, center.y, center.z, 1.0);

			// Texel snapping to eliminate sub-pixel crawling
			let world_units_per_texel = (2.0 * radius) / (SHADOW_MAP_SIZE as f32);
			let mut min_x = center_view.x - radius;
			let mut min_y = center_view.y - radius;

			min_x = (min_x / world_units_per_texel).floor() * world_units_per_texel;
			min_y = (min_y / world_units_per_texel).floor() * world_units_per_texel;
			let max_x = min_x + 2.0 * radius;
			let max_y = min_y + 2.0 * radius;

			let near_z = 0.0;
			let far_z = 2.0 * radius + FAR_OFFSET + 50.0;

			let light_proj = Mat4x4::orthographic(min_x, max_x, min_y, max_y, near_z, far_z);
			let view_proj = light_proj * light_view;

			set.cascades[c].view_proj = view_proj.to_array();
		}
	}

	pub fn cascade_view_proj(&self, light_idx: usize) -> [[f32; 16]; CASCADE_COUNT] {
		if light_idx < MAX_DIRECTIONAL_LIGHTS {
			if let Some(set) = &self.cascade_sets[light_idx] {
				return [
					set.cascades[0].view_proj,
					set.cascades[1].view_proj,
					set.cascades[2].view_proj,
					set.cascades[3].view_proj,
				];
			}
		}
		[Mat4x4::IDENTITY.to_array(); CASCADE_COUNT]
	}

	pub fn cascade_split_depths(&self) -> [f32; CASCADE_COUNT] {
		self.split_depths
	}

	pub fn cascade_shadow_ids(&self, light_idx: usize) -> [u32; CASCADE_COUNT] {
		if light_idx < MAX_DIRECTIONAL_LIGHTS {
			if let Some(set) = &self.cascade_sets[light_idx] {
				return [
					set.cascades[0].descriptor_id,
					set.cascades[1].descriptor_id,
					set.cascades[2].descriptor_id,
					set.cascades[3].descriptor_id,
				];
			}
		}
		[0; CASCADE_COUNT]
	}
}
