/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use crate::rhi;


static FULLSCREEN_VERT_SPV: &[u8] = include_bytes!("../../res/shaders/fullscreen.vert.spv");
static TAA_FRAG_SPV: &[u8] = include_bytes!("../../res/shaders/taa.frag.spv");
static TAA_SHARPEN_FRAG_SPV: &[u8] = include_bytes!("../../res/shaders/taa_sharpen.frag.spv");

pub const DEFAULT_TAA_BLEND_FACTOR: f32 = 0.08;
pub const DEFAULT_TAA_SHARPNESS: f32 = 0.25;

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct TaaPushConstants {
	pub rt_metrics: [f32; 4],
	pub color_tex_id: u32,
	pub history_tex_id: u32,
	pub velocity_tex_id: u32,
	pub depth_tex_id: u32,
	pub has_history: u32,
	pub blend_factor: f32,
	pub _pad: f32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct TaaSharpenPushConstants {
	pub rt_metrics: [f32; 4],
	pub texture_id: u32,
	pub sharpness: f32,
	pub _pad0: f32,
	pub _pad1: f32,
}

pub struct TaaTargets {
	pub width: u32,
	pub height: u32,
	pub history_images: [rhi::Image2; 2],
	pub history_views: [rhi::ImageView2; 2],
	pub history_descriptor_ids: [u32; 2],
}

pub struct TaaManager {
	pub linear_sampler: rhi::Sampler2,

	// Pass 1: TAA Temporal Resolve pipeline
	pub resolve_pipeline: rhi::Pipeline,
	pub resolve_meta: rhi::PipelineMeta,

	// Pass 2: Contrast Adaptive Sharpening (RCAS) pipeline
	pub sharpen_pipeline: rhi::Pipeline,
	pub sharpen_meta: rhi::PipelineMeta,

	pub swapchain_color_format: rhi::Format,
	pub has_history: bool,
	pub blend_factor: f32,
	pub sharpness: f32,

	pub targets: Option<TaaTargets>,
}

impl TaaManager {
	pub fn new(device: &rhi::Device, swapchain_color_format: rhi::Format) -> Self {
		crate::mochi_disp!("Initializing TAA (Temporal Anti-Aliasing) subsystem");

		let linear_sampler = device
			.create_sampler(
				rhi::SamplerFilter::Linear,
				rhi::SamplerAddressMode::ClampToEdge,
			)
			.expect("Failed to create TAA linear sampler");

		// 1. Pass 1 Pipeline: TAA Resolve
		let resolve_vert = rhi::Shader::new(
			device,
			rhi::ShaderStage::Vertex,
			FULLSCREEN_VERT_SPV,
			"main",
		);
		let resolve_frag = rhi::Shader::new(device, rhi::ShaderStage::Pixel, TAA_FRAG_SPV, "main");
		let resolve_meta = rhi::PipelineMeta::new_raw(
			rhi::ShaderStage::Pixel,
			std::mem::size_of::<TaaPushConstants>() as u32,
		);
		let resolve_pipeline = rhi::GraphicsPipelineBuilder::new(device, &resolve_meta)
			.shaders(&[&resolve_vert, &resolve_frag])
			.color_format(swapchain_color_format)
			.depth_write(false)
			.build();

		// 2. Pass 2 Pipeline: TAA Sharpen (RCAS)
		let sharpen_vert = rhi::Shader::new(
			device,
			rhi::ShaderStage::Vertex,
			FULLSCREEN_VERT_SPV,
			"main",
		);
		let sharpen_frag = rhi::Shader::new(
			device,
			rhi::ShaderStage::Pixel,
			TAA_SHARPEN_FRAG_SPV,
			"main",
		);
		let sharpen_meta = rhi::PipelineMeta::new_raw(
			rhi::ShaderStage::Pixel,
			std::mem::size_of::<TaaSharpenPushConstants>() as u32,
		);
		let sharpen_pipeline = rhi::GraphicsPipelineBuilder::new(device, &sharpen_meta)
			.shaders(&[&sharpen_vert, &sharpen_frag])
			.color_format(swapchain_color_format)
			.depth_write(false)
			.build();

		Self {
			linear_sampler,
			resolve_pipeline,
			resolve_meta,
			sharpen_pipeline,
			sharpen_meta,
			swapchain_color_format,
			has_history: false,
			blend_factor: DEFAULT_TAA_BLEND_FACTOR,
			sharpness: DEFAULT_TAA_SHARPNESS,
			targets: None,
		}
	}

	pub fn ensure_targets(
		&mut self,
		device: &rhi::Device,
		alloc: &rhi::Allocator,
		width: u32,
		height: u32,
	) {
		if let Some(targets) = &self.targets
			&& targets.width == width
			&& targets.height == height
		{
			return;
		}

		let (history_images, history_views, history_descriptor_ids) = Self::create_history_targets(
			device,
			alloc,
			width,
			height,
			self.swapchain_color_format,
			&self.linear_sampler,
		);

		self.has_history = false;
		self.targets = Some(TaaTargets {
			width,
			height,
			history_images,
			history_views,
			history_descriptor_ids,
		});
	}

	pub fn free_targets(&mut self) {
		self.targets = None;
		self.has_history = false;
	}

	fn create_history_targets(
		device: &rhi::Device,
		alloc: &rhi::Allocator,
		width: u32,
		height: u32,
		format: rhi::Format,
		sampler: &rhi::Sampler2,
	) -> ([rhi::Image2; 2], [rhi::ImageView2; 2], [u32; 2]) {
		let img0 = rhi::Image2::builder(width, height, format)
			.usage(rhi::ImageUsage::ColorAttachment | rhi::ImageUsage::Sampled)
			.build(alloc);
		let view0 = device
			.create_image_view(&img0, format)
			.expect("Failed to create TAA history image view 0");
		let desc0 = device.write_texture_descriptor(&view0, sampler);

		let img1 = rhi::Image2::builder(width, height, format)
			.usage(rhi::ImageUsage::ColorAttachment | rhi::ImageUsage::Sampled)
			.build(alloc);
		let view1 = device
			.create_image_view(&img1, format)
			.expect("Failed to create TAA history image view 1");
		let desc1 = device.write_texture_descriptor(&view1, sampler);

		([img0, img1], [view0, view1], [desc0, desc1])
	}
}


#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_taa_push_constants_layout() {
		assert_eq!(std::mem::size_of::<TaaPushConstants>(), 44);
		assert_eq!(std::mem::size_of::<TaaPushConstants>() % 4, 0);
	}

	#[test]
	fn test_taa_sharpen_push_constants_layout() {
		assert_eq!(std::mem::size_of::<TaaSharpenPushConstants>(), 32);
		assert_eq!(std::mem::size_of::<TaaSharpenPushConstants>() % 4, 0);
	}

	#[test]
	fn test_velocity_reprojection_math() {
		use mochi_math::{Mat4x4, Vec3, Vec4};

		let eye = Vec3::new(0.0, 2.2, 6.0);
		let target = Vec3::new(0.0, 2.2, 0.0);
		let up = Vec3::new(0.0, 1.0, 0.0);
		let view = Mat4x4::look_at(eye, target, up);

		let fov = 60.0;
		let aspect = 1280.0 / 720.0;
		let near = 0.1;
		let far = 1000.0;
		let mut proj0 = Mat4x4::perspective(fov, aspect, near, far);
		let mut proj1 = Mat4x4::perspective(fov, aspect, near, far);

		let j0 = (0.25f32, -0.25f32);
		let j1 = (-0.25f32, 0.25f32);

		let j_ndc_x0 = 2.0 * j0.0 / 1280.0;
		let j_ndc_y0 = 2.0 * j0.1 / 720.0;
		proj0.rows[0].z -= j_ndc_x0;
		proj0.rows[1].z -= j_ndc_y0;

		let j_ndc_x1 = 2.0 * j1.0 / 1280.0;
		let j_ndc_y1 = 2.0 * j1.1 / 720.0;
		proj1.rows[0].z -= j_ndc_x1;
		proj1.rows[1].z -= j_ndc_y1;

		let vp0 = proj0 * view;
		let vp1 = proj1 * view;
		let inv_vp1 = vp1.inverse();

		// Test a 3D point in the scene
		let world_pt = Vec3::new(0.0, 2.0, 0.9);
		let clip1 = vp1 * Vec4::new(world_pt.x, world_pt.y, world_pt.z, 1.0);
		let ndc1 = Vec3::new(clip1.x / clip1.w, clip1.y / clip1.w, clip1.z / clip1.w);

		let clip0 = vp0 * Vec4::new(world_pt.x, world_pt.y, world_pt.z, 1.0);
		let ndc0 = Vec3::new(clip0.x / clip0.w, clip0.y / clip0.w, clip0.z / clip0.w);

		// Now simulate velocity.frag:
		// in_uv corresponds to ndc1:
		let in_uv_x = ndc1.x * 0.5 + 0.5;
		let in_uv_y = ndc1.y * 0.5 + 0.5;
		let depth = ndc1.z;

		let reconstructed_clip = Vec4::new(ndc1.x, ndc1.y, depth, 1.0);
		let reconstructed_world = inv_vp1 * reconstructed_clip;
		let rw = reconstructed_world / reconstructed_world.w;

		assert!((rw.x - world_pt.x).abs() < 1e-4);
		assert!((rw.y - world_pt.y).abs() < 1e-4);
		assert!((rw.z - world_pt.z).abs() < 1e-4);

		let prev_clip = vp0 * Vec4::new(rw.x, rw.y, rw.z, 1.0);
		let prev_ndc = Vec3::new(
			prev_clip.x / prev_clip.w,
			prev_clip.y / prev_clip.w,
			prev_clip.z / prev_clip.w,
		);

		assert!((prev_ndc.x - ndc0.x).abs() < 1e-4);
		assert!((prev_ndc.y - ndc0.y).abs() < 1e-4);

		let curr_unjittered_ndc_x = ndc1.x - j_ndc_x1;
		let curr_unjittered_ndc_y = ndc1.y - j_ndc_y1;
		let prev_unjittered_ndc_x = prev_ndc.x - j_ndc_x0;
		let prev_unjittered_ndc_y = prev_ndc.y - j_ndc_y0;

		let velocity_x = (curr_unjittered_ndc_x - prev_unjittered_ndc_x) * 0.5;
		let velocity_y = (curr_unjittered_ndc_y - prev_unjittered_ndc_y) * 0.5;

		// Pure motion vector should be ZERO for static scene regardless of jitter!
		assert!(velocity_x.abs() < 1e-4, "velocity_x was {}", velocity_x);
		assert!(velocity_y.abs() < 1e-4, "velocity_y was {}", velocity_y);

		let prev_uv_x = in_uv_x - velocity_x;
		let prev_uv_y = in_uv_y - velocity_y;

		// For stationary scene, prev_uv must match in_uv exactly (no shaking!)
		assert!((prev_uv_x - in_uv_x).abs() < 1e-4);
		assert!((prev_uv_y - in_uv_y).abs() < 1e-4);
	}
}
