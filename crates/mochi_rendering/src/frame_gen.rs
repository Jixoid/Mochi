/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use mochi_rhi as rhi;
use mochi_except::*;


static FULLSCREEN_VERT_SPV: &[u8] = include_bytes!("../../../res/shaders/fullscreen.vert.spv");
static FSR3_FRAME_GEN_FRAG_SPV: &[u8] = include_bytes!("../../../res/shaders/fsr3_frame_gen.frag.spv");
static FULLSCREEN_BLIT_FRAG_SPV: &[u8] = include_bytes!("../../../res/shaders/fullscreen_blit.frag.spv");

pub const DEFAULT_INTERPOLATION_FACTOR: f32 = 0.5;
pub const DEFAULT_DISOCCLUSION_THRESHOLD: f32 = 0.08;

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Fsr3FrameGenPushConstants {
	pub display_metrics: [f32; 4],
	pub prev_color_tex_id: u32,
	pub curr_color_tex_id: u32,
	pub velocity_tex_id: u32,
	pub depth_tex_id: u32,
	pub interpolation_factor: f32,
	pub disocclusion_threshold: f32,
	pub has_history: u32,
	pub _pad0: f32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct FullscreenBlitPushConstants {
	pub texture_id: u32,
	pub _pad0: u32,
	pub _pad1: u32,
	pub _pad2: u32,
}

pub struct FsrFrameGenTargets {
	pub width: u32,
	pub height: u32,
	pub color_images: [rhi::Image2; 2],
	pub color_views: [rhi::ImageView2; 2],
	pub color_descriptor_ids: [u32; 2],
}

pub struct FsrFrameGenManager {
	pub linear_sampler: rhi::Sampler2,

	// Pipeline: AMD FSR 3.1 Optical Flow Frame Generation
	pub frame_gen_pipeline: rhi::Pipeline,
	pub frame_gen_meta: rhi::PipelineMeta,

	// Pipeline: Real Frame direct blit to swapchain
	pub blit_pipeline: rhi::Pipeline,
	pub blit_meta: rhi::PipelineMeta,

	pub swapchain_color_format: rhi::Format,
	pub has_history: bool,
	pub interpolation_factor: f32,
	pub disocclusion_threshold: f32,

	pub targets: Option<FsrFrameGenTargets>,
}

impl FsrFrameGenManager {
	pub fn new(device: &rhi::Device, swapchain_color_format: rhi::Format) -> Self {
		mochi_disp!("Initializing FidelityFX FSR 3.1 (Frame Generation) subsystem");

		let linear_sampler = device
			.create_sampler(
				rhi::SamplerFilter::Linear,
				rhi::SamplerAddressMode::ClampToEdge,
			)
			.expect("Failed to create FSR Frame Generation linear sampler");

		// 1. Frame Generation Pipeline
		let fg_vert = rhi::Shader::new(
			device,
			rhi::ShaderStage::Vertex,
			FULLSCREEN_VERT_SPV,
			"main",
		);
		let fg_frag = rhi::Shader::new(
			device,
			rhi::ShaderStage::Pixel,
			FSR3_FRAME_GEN_FRAG_SPV,
			"main",
		);
		let frame_gen_meta = rhi::PipelineMeta::new_raw(
			rhi::ShaderStage::Pixel,
			std::mem::size_of::<Fsr3FrameGenPushConstants>() as u32,
		);
		let frame_gen_pipeline = rhi::GraphicsPipelineBuilder::new(device, &frame_gen_meta)
			.shaders(&[&fg_vert, &fg_frag])
			.color_format(swapchain_color_format)
			.depth_write(false)
			.build();

		// 2. Real Frame Blit Pipeline
		let blit_vert = rhi::Shader::new(
			device,
			rhi::ShaderStage::Vertex,
			FULLSCREEN_VERT_SPV,
			"main",
		);
		let blit_frag = rhi::Shader::new(
			device,
			rhi::ShaderStage::Pixel,
			FULLSCREEN_BLIT_FRAG_SPV,
			"main",
		);
		let blit_meta = rhi::PipelineMeta::new_raw(
			rhi::ShaderStage::Pixel,
			std::mem::size_of::<FullscreenBlitPushConstants>() as u32,
		);
		let blit_pipeline = rhi::GraphicsPipelineBuilder::new(device, &blit_meta)
			.shaders(&[&blit_vert, &blit_frag])
			.color_format(swapchain_color_format)
			.depth_write(false)
			.build();

		Self {
			linear_sampler,
			frame_gen_pipeline,
			frame_gen_meta,
			blit_pipeline,
			blit_meta,
			swapchain_color_format,
			has_history: false,
			interpolation_factor: DEFAULT_INTERPOLATION_FACTOR,
			disocclusion_threshold: DEFAULT_DISOCCLUSION_THRESHOLD,
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

		let (color_images, color_views, color_descriptor_ids) = Self::create_history_targets(
			device,
			alloc,
			width,
			height,
			self.swapchain_color_format,
			&self.linear_sampler,
		);

		self.has_history = false;
		self.targets = Some(FsrFrameGenTargets {
			width,
			height,
			color_images,
			color_views,
			color_descriptor_ids,
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
			.expect("Failed to create Frame Gen history image view 0");
		let desc0 = device.write_texture_descriptor(&view0, sampler);

		let img1 = rhi::Image2::builder(width, height, format)
			.usage(rhi::ImageUsage::ColorAttachment | rhi::ImageUsage::Sampled)
			.build(alloc);
		let view1 = device
			.create_image_view(&img1, format)
			.expect("Failed to create Frame Gen history image view 1");
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

		let fg_vert = rhi::Shader::new(
			device,
			rhi::ShaderStage::Vertex,
			FULLSCREEN_VERT_SPV,
			"main",
		);
		let fg_frag = rhi::Shader::new(
			device,
			rhi::ShaderStage::Pixel,
			FSR3_FRAME_GEN_FRAG_SPV,
			"main",
		);
		self.frame_gen_pipeline = rhi::GraphicsPipelineBuilder::new(device, &self.frame_gen_meta)
			.shaders(&[&fg_vert, &fg_frag])
			.color_format(swapchain_color_format)
			.depth_write(false)
			.build();

		let blit_vert = rhi::Shader::new(
			device,
			rhi::ShaderStage::Vertex,
			FULLSCREEN_VERT_SPV,
			"main",
		);
		let blit_frag = rhi::Shader::new(
			device,
			rhi::ShaderStage::Pixel,
			FULLSCREEN_BLIT_FRAG_SPV,
			"main",
		);
		self.blit_pipeline = rhi::GraphicsPipelineBuilder::new(device, &self.blit_meta)
			.shaders(&[&blit_vert, &blit_frag])
			.color_format(swapchain_color_format)
			.depth_write(false)
			.build();
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_frame_gen_constants_layout() {
		assert_eq!(std::mem::size_of::<Fsr3FrameGenPushConstants>(), 48);
		assert_eq!(std::mem::size_of::<FullscreenBlitPushConstants>(), 16);
	}
}
