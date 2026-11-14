/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use crate::rhi;


static FULLSCREEN_VERT_SPV: &[u8] = include_bytes!("../../res/shaders/fullscreen.vert.spv");
static FSR3_UPSCALE_FRAG_SPV: &[u8] = include_bytes!("../../res/shaders/fsr3_upscale.frag.spv");
static FSR3_RCAS_FRAG_SPV: &[u8] = include_bytes!("../../res/shaders/taa_sharpen.frag.spv");

pub const DEFAULT_FSR_BLEND_FACTOR: f32 = 0.08;
pub const DEFAULT_FSR_SHARPNESS: f32 = 0.25;

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Fsr3UpscalePushConstants {
	pub render_metrics: [f32; 4],
	pub display_metrics: [f32; 4],
	pub jitter: [f32; 4],
	pub color_tex_id: u32,
	pub velocity_tex_id: u32,
	pub depth_tex_id: u32,
	pub history_tex_id: u32,
	pub has_history: u32,
	pub blend_factor: f32,
	pub _pad0: f32,
	pub _pad1: f32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct FsrRcasPushConstants {
	pub rt_metrics: [f32; 4],
	pub texture_id: u32,
	pub sharpness: f32,
	pub _pad0: f32,
	pub _pad1: f32,
}

pub struct FsrTargets {
	pub width: u32,
	pub height: u32,
	pub history_images: [rhi::Image2; 2],
	pub history_views: [rhi::ImageView2; 2],
	pub history_descriptor_ids: [u32; 2],
}

pub struct FsrManager {
	pub linear_sampler: rhi::Sampler2,

	// Pass 1: AMD FSR 3.1 Temporal Super Resolution Upscale pipeline
	pub upscale_pipeline: rhi::Pipeline,
	pub upscale_meta: rhi::PipelineMeta,

	// Pass 2: AMD RCAS (Robust Contrast Adaptive Sharpening) pipeline
	pub rcas_pipeline: rhi::Pipeline,
	pub rcas_meta: rhi::PipelineMeta,

	pub swapchain_color_format: rhi::Format,
	pub has_history: bool,
	pub blend_factor: f32,
	pub sharpness: f32,

	pub targets: Option<FsrTargets>,
}

impl FsrManager {
	pub fn new(device: &rhi::Device, swapchain_color_format: rhi::Format) -> Self {
		crate::mochi_disp!("Initializing FidelityFX FSR 3.1 (Temporal Super Resolution) subsystem");

		let linear_sampler = device
			.create_sampler(
				rhi::SamplerFilter::Linear,
				rhi::SamplerAddressMode::ClampToEdge,
			)
			.expect("Failed to create FSR linear sampler");

		// 1. Pass 1 Pipeline: FSR 3.1 Upscale
		let upscale_vert = rhi::Shader::new(
			device,
			rhi::ShaderStage::Vertex,
			FULLSCREEN_VERT_SPV,
			"main",
		);
		let upscale_frag =
			rhi::Shader::new(device, rhi::ShaderStage::Pixel, FSR3_UPSCALE_FRAG_SPV, "main");
		let upscale_meta = rhi::PipelineMeta::new_raw(
			rhi::ShaderStage::Pixel,
			std::mem::size_of::<Fsr3UpscalePushConstants>() as u32,
		);
		let upscale_pipeline = rhi::GraphicsPipelineBuilder::new(device, &upscale_meta)
			.shaders(&[&upscale_vert, &upscale_frag])
			.color_format(swapchain_color_format)
			.depth_write(false)
			.build();

		// 2. Pass 2 Pipeline: RCAS
		let rcas_vert = rhi::Shader::new(
			device,
			rhi::ShaderStage::Vertex,
			FULLSCREEN_VERT_SPV,
			"main",
		);
		let rcas_frag = rhi::Shader::new(device, rhi::ShaderStage::Pixel, FSR3_RCAS_FRAG_SPV, "main");
		let rcas_meta = rhi::PipelineMeta::new_raw(
			rhi::ShaderStage::Pixel,
			std::mem::size_of::<FsrRcasPushConstants>() as u32,
		);
		let rcas_pipeline = rhi::GraphicsPipelineBuilder::new(device, &rcas_meta)
			.shaders(&[&rcas_vert, &rcas_frag])
			.color_format(swapchain_color_format)
			.depth_write(false)
			.build();

		Self {
			linear_sampler,
			upscale_pipeline,
			upscale_meta,
			rcas_pipeline,
			rcas_meta,
			swapchain_color_format,
			has_history: false,
			blend_factor: DEFAULT_FSR_BLEND_FACTOR,
			sharpness: DEFAULT_FSR_SHARPNESS,
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

		let (history_images, history_views, history_descriptor_ids) =
			Self::create_history_targets(
				device,
				alloc,
				width,
				height,
				self.swapchain_color_format,
				&self.linear_sampler,
			);

		self.has_history = false;
		self.targets = Some(FsrTargets {
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
			.expect("Failed to create FSR history image view 0");
		let desc0 = device.write_texture_descriptor(&view0, sampler);

		let img1 = rhi::Image2::builder(width, height, format)
			.usage(rhi::ImageUsage::ColorAttachment | rhi::ImageUsage::Sampled)
			.build(alloc);
		let view1 = device
			.create_image_view(&img1, format)
			.expect("Failed to create FSR history image view 1");
		let desc1 = device.write_texture_descriptor(&view1, sampler);

		([img0, img1], [view0, view1], [desc0, desc1])
	}

	pub fn update_swapchain_format(
		&mut self,
		device: &rhi::Device,
		swapchain_color_format: rhi::Format,
	) {
		if self.swapchain_color_format == swapchain_color_format {
			return;
		}

		self.swapchain_color_format = swapchain_color_format;
		self.targets = None;
		self.has_history = false;

		let upscale_vert = rhi::Shader::new(
			device,
			rhi::ShaderStage::Vertex,
			FULLSCREEN_VERT_SPV,
			"main",
		);
		let upscale_frag =
			rhi::Shader::new(device, rhi::ShaderStage::Pixel, FSR3_UPSCALE_FRAG_SPV, "main");
		self.upscale_pipeline = rhi::GraphicsPipelineBuilder::new(device, &self.upscale_meta)
			.shaders(&[&upscale_vert, &upscale_frag])
			.color_format(swapchain_color_format)
			.depth_write(false)
			.build();

		let rcas_vert = rhi::Shader::new(
			device,
			rhi::ShaderStage::Vertex,
			FULLSCREEN_VERT_SPV,
			"main",
		);
		let rcas_frag = rhi::Shader::new(device, rhi::ShaderStage::Pixel, FSR3_RCAS_FRAG_SPV, "main");
		self.rcas_pipeline = rhi::GraphicsPipelineBuilder::new(device, &self.rcas_meta)
			.shaders(&[&rcas_vert, &rcas_frag])
			.color_format(swapchain_color_format)
			.depth_write(false)
			.build();
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_fsr_constants_layout() {
		assert_eq!(std::mem::size_of::<Fsr3UpscalePushConstants>(), 80);
		assert_eq!(std::mem::size_of::<FsrRcasPushConstants>(), 32);
	}
}
