/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use crate::rhi;


static SMAA_AREA_BYTES: &[u8] = include_bytes!("../../res/textures/smaa_area.bin");
static SMAA_SEARCH_BYTES: &[u8] = include_bytes!("../../res/textures/smaa_search.bin");

static SMAA_EDGE_VERT_SPV: &[u8] = include_bytes!("../../res/shaders/smaa_edge.vert.spv");
static SMAA_EDGE_FRAG_SPV: &[u8] = include_bytes!("../../res/shaders/smaa_edge.frag.spv");
static SMAA_BLEND_VERT_SPV: &[u8] = include_bytes!("../../res/shaders/smaa_blend.vert.spv");
static SMAA_BLEND_FRAG_SPV: &[u8] = include_bytes!("../../res/shaders/smaa_blend.frag.spv");
static SMAA_NEIGHBOR_VERT_SPV: &[u8] = include_bytes!("../../res/shaders/smaa_neighbor.vert.spv");
static SMAA_NEIGHBOR_FRAG_SPV: &[u8] = include_bytes!("../../res/shaders/smaa_neighbor.frag.spv");
static SMAA_NEIGHBOR_REPROJECT_FRAG_SPV: &[u8] =
	include_bytes!("../../res/shaders/smaa_neighbor_reproject.frag.spv");
static SMAA_RESOLVE_VERT_SPV: &[u8] = include_bytes!("../../res/shaders/smaa_resolve.vert.spv");
static SMAA_RESOLVE_FRAG_SPV: &[u8] = include_bytes!("../../res/shaders/smaa_resolve.frag.spv");

pub const AREATEX_WIDTH: u32 = 160;
pub const AREATEX_HEIGHT: u32 = 560;

pub const SEARCHTEX_WIDTH: u32 = 64;
pub const SEARCHTEX_HEIGHT: u32 = 16;


#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct SmaaEdgePushConstants {
	pub rt_metrics: [f32; 4],
	pub color_tex_id: u32,
	pub _pad0: u32,
	pub _pad1: u32,
	pub _pad2: u32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct SmaaBlendPushConstants {
	pub rt_metrics: [f32; 4],
	pub subsample_indices: [f32; 4],
	pub edges_tex_id: u32,
	pub area_tex_id: u32,
	pub search_tex_id: u32,
	pub _pad: u32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct SmaaNeighborPushConstants {
	pub rt_metrics: [f32; 4],
	pub color_tex_id: u32,
	pub blend_tex_id: u32,
	pub velocity_tex_id: u32,
	pub _pad: u32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct SmaaResolvePushConstants {
	pub rt_metrics: [f32; 4],
	pub current_color_tex_id: u32,
	pub previous_color_tex_id: u32,
	pub velocity_tex_id: u32,
	pub has_history: u32,
}

pub struct SmaaTargets {
	pub width: u32,
	pub height: u32,

	// Pass 1: Edges target (R8G8_UNORM)
	pub edges_image: rhi::Image2,
	pub edges_view: rhi::ImageView2,
	pub edges_descriptor_id: u32,

	// Pass 2: Blending weights target (RGBA8_UNORM)
	pub blend_image: rhi::Image2,
	pub blend_view: rhi::ImageView2,
	pub blend_descriptor_id: u32,

	// Pass 3 (Temporal): History targets (ping-pong)
	pub history_images: Option<[rhi::Image2; 2]>,
	pub history_views: Option<[rhi::ImageView2; 2]>,
	pub history_descriptor_ids: Option<[u32; 2]>,
}

pub struct SmaaManager {
	// Precomputed lookup textures
	pub area_image: rhi::Image2,
	pub area_view: rhi::ImageView2,
	pub area_descriptor_id: u32,

	pub search_image: rhi::Image2,
	pub search_view: rhi::ImageView2,
	pub search_descriptor_id: u32,

	pub linear_sampler: rhi::Sampler2,
	pub point_sampler: rhi::Sampler2,

	// Pass 1: Edge detection pipeline
	pub edge_pipeline: rhi::Pipeline,
	pub edge_meta: rhi::PipelineMeta,

	// Pass 2: Blending weights calculation pipeline
	pub blend_pipeline: rhi::Pipeline,
	pub blend_meta: rhi::PipelineMeta,

	// Pass 3: Neighborhood blending pipeline (non-reprojected)
	pub neighbor_pipeline: rhi::Pipeline,
	pub neighbor_meta: rhi::PipelineMeta,

	// Pass 3: Neighborhood blending pipeline (reprojected with velocity in alpha)
	pub neighbor_reproject_pipeline: rhi::Pipeline,
	pub neighbor_reproject_meta: rhi::PipelineMeta,

	// Pass 4: Temporal resolve pipeline
	pub resolve_pipeline: rhi::Pipeline,
	pub resolve_meta: rhi::PipelineMeta,

	pub swapchain_color_format: rhi::Format,
	pub has_history: bool,

	// Dynamic targets
	pub targets: Option<SmaaTargets>,
}

impl SmaaManager {
	pub fn new(
		device: &rhi::Device,
		alloc: &rhi::Allocator,
		cmdmng: &rhi::CommandManager,
		swapchain_color_format: rhi::Format,
	) -> Self {
		crate::mochi_disp!("Initializing SMAA anti-aliasing subsystem");

		let linear_sampler = device
			.create_sampler(
				rhi::SamplerFilter::Linear,
				rhi::SamplerAddressMode::ClampToEdge,
			)
			.expect("Failed to create SMAA linear sampler");

		let point_sampler = device
			.create_sampler(
				rhi::SamplerFilter::Nearest,
				rhi::SamplerAddressMode::ClampToEdge,
			)
			.expect("Failed to create SMAA point sampler");

		// 1. Upload precomputed AreaTex (160x560 R8G8_UNORM)
		let area_image = rhi::Image2::builder(AREATEX_WIDTH, AREATEX_HEIGHT, rhi::Format::V2Norm8U)
			.usage(rhi::ImageUsage::Sampled | rhi::ImageUsage::TransferDst)
			.build(alloc);

		device
			.upload_image_data(
				alloc,
				cmdmng.raw(),
				&area_image,
				AREATEX_WIDTH,
				AREATEX_HEIGHT,
				SMAA_AREA_BYTES,
			)
			.expect("Failed to upload SMAA AreaTex");
		let area_view = device
			.create_image_view(&area_image, rhi::Format::V2Norm8U)
			.expect("Failed to create SMAA AreaTex image view");
		let area_descriptor_id = device.write_texture_descriptor(&area_view, &linear_sampler);

		// 2. Upload precomputed SearchTex (64x16 R8_UNORM)
		let search_image = rhi::Image2::builder(SEARCHTEX_WIDTH, SEARCHTEX_HEIGHT, rhi::Format::Norm8U)
			.usage(rhi::ImageUsage::Sampled | rhi::ImageUsage::TransferDst)
			.build(alloc);

		device
			.upload_image_data(
				alloc,
				cmdmng.raw(),
				&search_image,
				SEARCHTEX_WIDTH,
				SEARCHTEX_HEIGHT,
				SMAA_SEARCH_BYTES,
			)
			.expect("Failed to upload SMAA SearchTex");
		let search_view = device
			.create_image_view(&search_image, rhi::Format::Norm8U)
			.expect("Failed to create SMAA SearchTex image view");
		let search_descriptor_id = device.write_texture_descriptor(&search_view, &point_sampler);

		// 3. Create Pass 1 Pipeline (Edge Detection)
		let edge_vert = rhi::Shader::new(device, rhi::ShaderStage::Vertex, SMAA_EDGE_VERT_SPV, "main");
		let edge_frag = rhi::Shader::new(device, rhi::ShaderStage::Pixel, SMAA_EDGE_FRAG_SPV, "main");
		let edge_meta = rhi::PipelineMeta::new_raw(
			rhi::ShaderStage::Vertex | rhi::ShaderStage::Pixel,
			std::mem::size_of::<SmaaEdgePushConstants>() as u32,
		);
		let edge_pipeline = rhi::Pipeline::new_graphics(
			device,
			&edge_meta,
			&[&edge_vert, &edge_frag],
			rhi::PolygonMode::Fill,
			rhi::PrimitiveTopology::TriangleList,
			rhi::Format::V2Norm8U,
			rhi::Format::Undefined,
			false,
			false,
			rhi::MsaaSamples::Off,
		);

		// 4. Create Pass 2 Pipeline (Blending Weights)
		let blend_vert = rhi::Shader::new(
			device,
			rhi::ShaderStage::Vertex,
			SMAA_BLEND_VERT_SPV,
			"main",
		);
		let blend_frag = rhi::Shader::new(device, rhi::ShaderStage::Pixel, SMAA_BLEND_FRAG_SPV, "main");
		let blend_meta = rhi::PipelineMeta::new_raw(
			rhi::ShaderStage::Vertex | rhi::ShaderStage::Pixel,
			std::mem::size_of::<SmaaBlendPushConstants>() as u32,
		);
		let blend_pipeline = rhi::Pipeline::new_graphics(
			device,
			&blend_meta,
			&[&blend_vert, &blend_frag],
			rhi::PolygonMode::Fill,
			rhi::PrimitiveTopology::TriangleList,
			rhi::Format::V4Norm8U,
			rhi::Format::Undefined,
			false,
			false,
			rhi::MsaaSamples::Off,
		);

		// 5. Create Pass 3 Pipeline (Neighborhood Blending - Non-reprojected)
		let neighbor_vert = rhi::Shader::new(
			device,
			rhi::ShaderStage::Vertex,
			SMAA_NEIGHBOR_VERT_SPV,
			"main",
		);
		let neighbor_frag = rhi::Shader::new(
			device,
			rhi::ShaderStage::Pixel,
			SMAA_NEIGHBOR_FRAG_SPV,
			"main",
		);
		let neighbor_meta = rhi::PipelineMeta::new_raw(
			rhi::ShaderStage::Vertex | rhi::ShaderStage::Pixel,
			std::mem::size_of::<SmaaNeighborPushConstants>() as u32,
		);
		let neighbor_pipeline = rhi::Pipeline::new_graphics(
			device,
			&neighbor_meta,
			&[&neighbor_vert, &neighbor_frag],
			rhi::PolygonMode::Fill,
			rhi::PrimitiveTopology::TriangleList,
			swapchain_color_format,
			rhi::Format::Undefined,
			false,
			false,
			rhi::MsaaSamples::Off,
		);

		// 6. Create Pass 3 Pipeline (Neighborhood Blending - Reprojected with Velocity)
		let neighbor_reproject_frag = rhi::Shader::new(
			device,
			rhi::ShaderStage::Pixel,
			SMAA_NEIGHBOR_REPROJECT_FRAG_SPV,
			"main",
		);
		let neighbor_reproject_meta = rhi::PipelineMeta::new_raw(
			rhi::ShaderStage::Vertex | rhi::ShaderStage::Pixel,
			std::mem::size_of::<SmaaNeighborPushConstants>() as u32,
		);
		let neighbor_reproject_pipeline = rhi::Pipeline::new_graphics(
			device,
			&neighbor_reproject_meta,
			&[&neighbor_vert, &neighbor_reproject_frag],
			rhi::PolygonMode::Fill,
			rhi::PrimitiveTopology::TriangleList,
			swapchain_color_format,
			rhi::Format::Undefined,
			false,
			false,
			rhi::MsaaSamples::Off,
		);

		// 7. Create Pass 4 Pipeline (Temporal Resolve)
		let resolve_vert = rhi::Shader::new(
			device,
			rhi::ShaderStage::Vertex,
			SMAA_RESOLVE_VERT_SPV,
			"main",
		);
		let resolve_frag = rhi::Shader::new(
			device,
			rhi::ShaderStage::Pixel,
			SMAA_RESOLVE_FRAG_SPV,
			"main",
		);
		let resolve_meta = rhi::PipelineMeta::new_raw(
			rhi::ShaderStage::Pixel,
			std::mem::size_of::<SmaaResolvePushConstants>() as u32,
		);
		let resolve_pipeline = rhi::Pipeline::new_graphics(
			device,
			&resolve_meta,
			&[&resolve_vert, &resolve_frag],
			rhi::PolygonMode::Fill,
			rhi::PrimitiveTopology::TriangleList,
			swapchain_color_format,
			rhi::Format::Undefined,
			false,
			false,
			rhi::MsaaSamples::Off,
		);

		crate::mochi_disp!("SMAA subsystems and pipelines initialized successfully");

		Self {
			area_image,
			area_view,
			area_descriptor_id,
			search_image,
			search_view,
			search_descriptor_id,
			linear_sampler,
			point_sampler,
			edge_pipeline,
			edge_meta,
			blend_pipeline,
			blend_meta,
			neighbor_pipeline,
			neighbor_meta,
			neighbor_reproject_pipeline,
			neighbor_reproject_meta,
			resolve_pipeline,
			resolve_meta,
			swapchain_color_format,
			has_history: false,
			targets: None,
		}
	}

	pub fn ensure_targets(
		&mut self,
		device: &rhi::Device,
		alloc: &rhi::Allocator,
		width: u32,
		height: u32,
		needs_history: bool,
	) {
		if let Some(targets) = &mut self.targets {
			if targets.width == width && targets.height == height {
				if needs_history && targets.history_images.is_none() {
					let (imgs, views, descs) = Self::create_history_targets(
						device,
						alloc,
						width,
						height,
						self.swapchain_color_format,
						&self.linear_sampler,
					);
					targets.history_images = Some(imgs);
					targets.history_views = Some(views);
					targets.history_descriptor_ids = Some(descs);
					self.has_history = false;
				} else if !needs_history && targets.history_images.is_some() {
					targets.history_images = None;
					targets.history_views = None;
					targets.history_descriptor_ids = None;
					self.has_history = false;
				}
				return;
			}
		}

		// Pass 1 target: Edges (R8G8_UNORM)
		let edges_image = rhi::Image2::builder(width, height, rhi::Format::V2Norm8U)
			.usage(rhi::ImageUsage::ColorAttachment | rhi::ImageUsage::Sampled)
			.build(alloc);

		let edges_view = device
			.create_image_view(&edges_image, rhi::Format::V2Norm8U)
			.expect("Failed to create SMAA edges image view");
		let edges_descriptor_id = device.write_texture_descriptor(&edges_view, &self.linear_sampler);

		// Pass 2 target: Blending Weights (RGBA8_UNORM)
		let blend_image = rhi::Image2::builder(width, height, rhi::Format::V4Norm8U)
			.usage(rhi::ImageUsage::ColorAttachment | rhi::ImageUsage::Sampled)
			.build(alloc);

		let blend_view = device
			.create_image_view(&blend_image, rhi::Format::V4Norm8U)
			.expect("Failed to create SMAA blend image view");
		let blend_descriptor_id = device.write_texture_descriptor(&blend_view, &self.linear_sampler);

		let (history_images, history_views, history_descriptor_ids) = if needs_history {
			let (imgs, views, descs) = Self::create_history_targets(
				device,
				alloc,
				width,
				height,
				self.swapchain_color_format,
				&self.linear_sampler,
			);
			(Some(imgs), Some(views), Some(descs))
		} else {
			(None, None, None)
		};

		self.has_history = false;
		self.targets = Some(SmaaTargets {
			width,
			height,
			edges_image,
			edges_view,
			edges_descriptor_id,
			blend_image,
			blend_view,
			blend_descriptor_id,
			history_images,
			history_views,
			history_descriptor_ids,
		});
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
			.expect("Failed to create SMAA history image view 0");
		let desc0 = device.write_texture_descriptor(&view0, sampler);

		let img1 = rhi::Image2::builder(width, height, format)
			.usage(rhi::ImageUsage::ColorAttachment | rhi::ImageUsage::Sampled)
			.build(alloc);
		let view1 = device
			.create_image_view(&img1, format)
			.expect("Failed to create SMAA history image view 1");
		let desc1 = device.write_texture_descriptor(&view1, sampler);

		([img0, img1], [view0, view1], [desc0, desc1])
	}
}
