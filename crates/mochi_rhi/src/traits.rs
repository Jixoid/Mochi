/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use mochi_math::Extent;

use crate::error::RhiResult;
use crate::types::*;


/// Central backend trait representing an RHI driver implementation
pub trait RhiBackend: 'static + Sized + Send + Sync {
	type Device: RhiDevice<Self>;
	type Allocator: RhiAllocator<Self>;
	type Buffer: RhiBuffer;
	type Image: RhiImage;
	type ImageView: RhiImageView;
	type Sampler: RhiSampler;
	type Shader: RhiShader;
	type Pipeline: RhiPipeline;
	type PipelineMeta: RhiPipelineMeta;
	type CommandBuffer: RhiCommandBuffer<Self>;
	type CommandQueue: RhiCommandQueue<Self>;
	type Swapchain: RhiSwapchain<Self>;
	type RenderTarget: RhiRenderTarget;
	type Semaphore: Send + Sync + Clone;
	type Fence: Send + Sync + Clone;

	fn create_device(app_name: &str, major: u16, minor: u16, patch: u16, build: u16) -> RhiResult<Self::Device>;
	fn create_allocator(device: &Self::Device) -> RhiResult<Self::Allocator>;
	fn create_command_queue(device: &Self::Device) -> RhiResult<Self::CommandQueue>;
	fn create_swapchain(device: &Self::Device, surface: u64, width: u32, height: u32) -> RhiResult<Self::Swapchain>;
}


/// Abstract Device interface
pub trait RhiDevice<B: RhiBackend>: Send + Sync {
	fn wait_idle(&self) -> RhiResult<()>;
	fn main_queue_family(&self) -> u32;

	fn create_semaphore(&self) -> RhiResult<B::Semaphore>;
	fn create_fence(&self, signaled: bool) -> RhiResult<B::Fence>;
	fn wait_fence(&self, fence: &B::Fence, timeout_ns: u64) -> RhiResult<()>;
	fn reset_fence(&self, fence: &B::Fence) -> RhiResult<()>;

	fn create_surface<W: HasWindowHandle + HasDisplayHandle>(&self, window: &W) -> RhiResult<u64>;
	fn destroy_surface(&self, surface_handle: u64);

	fn create_shader(&self, stage: ShaderStage, code: &[u8], entry: &str) -> RhiResult<B::Shader>;
	fn create_pipeline_meta(&self, push_stages: ShaderStage, push_size: u32) -> RhiResult<B::PipelineMeta>;
	fn create_graphics_pipeline(
		&self,
		meta: &B::PipelineMeta,
		shaders: &[&B::Shader],
		polymode: PolygonMode,
		primitive_topology: PrimitiveTopology,
		color_format: Format,
		depth_format: Format,
	) -> RhiResult<B::Pipeline>;

	fn init_descriptor_heap(&self, allocator: &B::Allocator) -> RhiResult<()>;
	fn fini_descriptor_heap(&self);
	fn write_texture_descriptor(&self, view: &B::ImageView, sampler: &B::Sampler) -> u32;
	fn descriptor_heap(&self) -> Option<B::Buffer>;
	fn sampler_heap(&self) -> Option<B::Buffer>;

	fn create_image_view(&self, image: &B::Image, format: Format) -> RhiResult<B::ImageView>;
	fn create_sampler(&self, filter: SamplerFilter, address_mode: SamplerAddressMode) -> RhiResult<B::Sampler>;
	fn upload_image_data(
		&self,
		alloc: &B::Allocator,
		queue: &B::CommandQueue,
		image: &B::Image,
		width: u32,
		height: u32,
		data: &[u8],
	) -> RhiResult<()>;
}


/// Abstract Memory Allocator interface
pub trait RhiAllocator<B: RhiBackend>: Send + Sync {
	fn alloc_buffer(
		&self,
		size: u64,
		usage: BufferUsage,
		create: AllocationCreate,
		location: AllocationLocation,
	) -> RhiResult<B::Buffer>;

	fn alloc_image(
		&self,
		ext: Extent<2, u32>,
		format: Format,
		usage: ImageUsage,
		tiling: ImageTiling,
		create: AllocationCreate,
		location: AllocationLocation,
	) -> RhiResult<B::Image>;

	fn alloc_sampler(
		&self,
		address_mode: Extent<3, SamplerAddressMode>,
		mag_filter: SamplerFilter,
		min_filter: SamplerFilter,
	) -> RhiResult<B::Sampler>;
}


/// Abstract GPU Buffer
pub trait RhiBuffer: Send + Sync {
	fn size(&self) -> u64;
	fn device_address(&self) -> u64;
	fn map(&mut self) -> RhiResult<*mut u8>;
	fn unmap(&mut self);
	fn flush(&mut self, offset: u64, size: u64) -> RhiResult<()>;
}


/// Abstract 2D Image
pub trait RhiImage: Send + Sync {
	fn ext(&self) -> Extent<2, u32>;
	fn format(&self) -> Format;
}


/// Abstract Image View
pub trait RhiImageView: Send + Sync {}


/// Abstract Texture Sampler
pub trait RhiSampler: Send + Sync {}


/// Abstract Shader Module
pub trait RhiShader: Send + Sync {
	fn stage(&self) -> ShaderStage;
	fn entry(&self) -> &str;
}


/// Abstract Pipeline Meta (Layout / Push constant metadata)
pub trait RhiPipelineMeta: Send + Sync {}


/// Abstract Pipeline (Graphics / Compute)
pub trait RhiPipeline: Send + Sync {
	fn kind(&self) -> PipelineKind;
}


/// Abstract Render Target (Framebuffer / Dynamic Rendering Attachment)
pub trait RhiRenderTarget: Send + Sync + Clone {
	fn color_format(&self) -> Format;
	fn depth_format(&self) -> Format;
}


/// Abstract Swapchain
pub trait RhiSwapchain<B: RhiBackend>: Send + Sync {
	fn resize(&mut self, width: u32, height: u32) -> RhiResult<()>;
	fn acquire_next_image(&mut self, signal_sem: Option<&B::Semaphore>) -> RhiResult<u32>;
	fn present(&mut self, image_index: u32, wait_sem: Option<&B::Semaphore>) -> RhiResult<()>;
	fn image_count(&self) -> u32;
	fn render_target(&self, index: u32) -> Option<&B::RenderTarget>;
	fn render_finished_semaphore(&self, index: u32) -> &B::Semaphore;
}


/// Abstract Command Buffer
pub trait RhiCommandBuffer<B: RhiBackend>: Send + Sync {
	fn begin(&mut self) -> RhiResult<()>;
	fn end(&mut self) -> RhiResult<()>;

	fn set_viewport(&mut self, vp: Viewport);
	fn set_scissor(&mut self, scissor: Rect2D);
	fn bind_pipeline(&mut self, pipeline: &B::Pipeline);
	fn bind_descriptor_heap(&mut self, res_heap: Option<&B::Buffer>, samp_heap: Option<&B::Buffer>, res_sz: u64, samp_sz: u64);
	fn push_constant(&mut self, stages: ShaderStage, offset: u32, bytes: &[u8]);
	fn bind_index_buffer(&mut self, buffer: &B::Buffer, offset: u64, index_type: IndexType);
	fn draw(&mut self, vertex_count: u32, instance_count: u32, first_vertex: u32, first_instance: u32);
	fn draw_indexed(&mut self, index_count: u32, instance_count: u32, first_index: u32, vertex_offset: i32, first_instance: u32);

	fn begin_rendering(&mut self, target: &B::RenderTarget, clear_color: ClearColor);
	fn end_rendering(&mut self, target: &B::RenderTarget);
}


/// Abstract Command Queue / Submission Manager
pub trait RhiCommandQueue<B: RhiBackend>: Send + Sync {
	fn allocate_graphic_command(&self) -> RhiResult<B::CommandBuffer>;
	fn submit(&self, cmd: &B::CommandBuffer) -> RhiResult<()>;
	fn submit_with_sync(&self, cmd: &B::CommandBuffer, wait_semaphore: Option<&B::Semaphore>, signal_semaphore: Option<&B::Semaphore>, fence: Option<&B::Fence>) -> RhiResult<()>;
}
