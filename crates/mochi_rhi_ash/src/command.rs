/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::sync::{Arc, Mutex};
use ash::vk;

use crate::buffer::AshBuffer;
use crate::pipeline::AshPipeline;
use crate::swapchain::AshRenderTarget;
use crate::sync::{AshFence, AshSemaphore};
use crate::types::{to_vk_index_type, to_vk_pipeline_bind_point, to_vk_shader_stage};
use crate::AshVulkanBackend;
use mochi_rhi::error::{RhiError, RhiResult};
use mochi_rhi::shader::ShaderStage;
use mochi_rhi::traits::{RhiCommandBuffer, RhiCommandQueue};
use mochi_rhi::types::{ClearColor, IndexType, Rect2D, Viewport};


pub struct AshCommandBuffer {
	device: ash::Device,
	handle: vk::CommandBuffer,
	current_pipeline_layout: vk::PipelineLayout,
}

unsafe impl Send for AshCommandBuffer {}
unsafe impl Sync for AshCommandBuffer {}

impl AshCommandBuffer {
	pub fn new(device: &ash::Device, handle: vk::CommandBuffer) -> Self {
		Self {
			device: device.clone(),
			handle,
			current_pipeline_layout: vk::PipelineLayout::null(),
		}
	}

	pub fn handle(&self) -> vk::CommandBuffer {
		self.handle
	}
}

impl RhiCommandBuffer<AshVulkanBackend> for AshCommandBuffer {
	fn begin(&mut self) -> RhiResult<()> {
		let begin_info = vk::CommandBufferBeginInfo::default()
			.flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

		unsafe {
			self.device
				.begin_command_buffer(self.handle, &begin_info)
				.map_err(|e| RhiError::BackendError(format!("Failed to begin command buffer: {e}")))?;
		}
		Ok(())
	}

	fn end(&mut self) -> RhiResult<()> {
		unsafe {
			self.device
				.end_command_buffer(self.handle)
				.map_err(|e| RhiError::BackendError(format!("Failed to end command buffer: {e}")))?;
		}
		Ok(())
	}

	fn set_viewport(&mut self, vp: Viewport) {
		let viewport = vk::Viewport {
			x: vp.x,
			y: vp.y,
			width: vp.width,
			height: vp.height,
			min_depth: vp.min_depth,
			max_depth: vp.max_depth,
		};
		unsafe {
			self.device.cmd_set_viewport(self.handle, 0, &[viewport]);
		}
	}

	fn set_scissor(&mut self, s: Rect2D) {
		let scissor = vk::Rect2D {
			offset: vk::Offset2D { x: s.x, y: s.y },
			extent: vk::Extent2D {
				width: s.width,
				height: s.height,
			},
		};
		unsafe {
			self.device.cmd_set_scissor(self.handle, 0, &[scissor]);
		}
	}

	fn bind_pipeline(&mut self, pipeline: &AshPipeline) {
		self.current_pipeline_layout = pipeline.layout();
		let bind_point = to_vk_pipeline_bind_point(pipeline.kind());
		unsafe {
			self.device.cmd_bind_pipeline(
				self.handle,
				bind_point,
				pipeline.handle(),
			);
			if let Some(set) = pipeline.descriptor_set() {
				self.device.cmd_bind_descriptor_sets(
					self.handle,
					bind_point,
					pipeline.layout(),
					0,
					&[set],
					&[],
				);
			}
		}
	}

	fn bind_descriptor_heap(
		&mut self,
		_res_heap: Option<&AshBuffer>,
		_samp_heap: Option<&AshBuffer>,
		_res_sz: u64,
		_samp_sz: u64,
	) {
		// Descriptor buffers/heaps are bound via BDA or descriptor extensions
	}

	fn push_constant(&mut self, stages: ShaderStage, offset: u32, bytes: &[u8]) {
		if self.current_pipeline_layout != vk::PipelineLayout::null() && !bytes.is_empty() {
			unsafe {
				self.device.cmd_push_constants(
					self.handle,
					self.current_pipeline_layout,
					to_vk_shader_stage(stages),
					offset,
					bytes,
				);
			}
		}
	}

	fn bind_index_buffer(&mut self, buffer: &AshBuffer, offset: u64, index_type: IndexType) {
		unsafe {
			self.device.cmd_bind_index_buffer(
				self.handle,
				buffer.handle(),
				offset,
				to_vk_index_type(index_type),
			);
		}
	}

	fn draw(&mut self, vertex_count: u32, instance_count: u32, first_vertex: u32, first_instance: u32) {
		unsafe {
			self.device.cmd_draw(
				self.handle,
				vertex_count,
				instance_count,
				first_vertex,
				first_instance,
			);
		}
	}

	fn draw_indexed(
		&mut self,
		index_count: u32,
		instance_count: u32,
		first_index: u32,
		vertex_offset: i32,
		first_instance: u32,
	) {
		unsafe {
			self.device.cmd_draw_indexed(
				self.handle,
				index_count,
				instance_count,
				first_index,
				vertex_offset,
				first_instance,
			);
		}
	}

	fn begin_rendering(&mut self, target: &AshRenderTarget, clear_color: ClearColor) {
		// 1. Image barriers: Color to COLOR_ATTACHMENT_OPTIMAL, Depth to DEPTH_ATTACHMENT_OPTIMAL
		let mut barriers = Vec::with_capacity(3);

		let color_barrier = vk::ImageMemoryBarrier2::default()
			.src_stage_mask(vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT)
			.src_access_mask(vk::AccessFlags2::NONE)
			.dst_stage_mask(vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT)
			.dst_access_mask(vk::AccessFlags2::COLOR_ATTACHMENT_WRITE)
			.old_layout(vk::ImageLayout::UNDEFINED)
			.new_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
			.image(target.color_image)
			.subresource_range(
				vk::ImageSubresourceRange::default()
					.aspect_mask(vk::ImageAspectFlags::COLOR)
					.base_mip_level(0)
					.level_count(1)
					.base_array_layer(0)
					.layer_count(1),
			);
		barriers.push(color_barrier);

		if target.resolve_image != vk::Image::null() {
			let resolve_barrier = vk::ImageMemoryBarrier2::default()
				.src_stage_mask(vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT)
				.src_access_mask(vk::AccessFlags2::NONE)
				.dst_stage_mask(vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT)
				.dst_access_mask(vk::AccessFlags2::COLOR_ATTACHMENT_WRITE)
				.old_layout(vk::ImageLayout::UNDEFINED)
				.new_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
				.image(target.resolve_image)
				.subresource_range(
					vk::ImageSubresourceRange::default()
						.aspect_mask(vk::ImageAspectFlags::COLOR)
						.base_mip_level(0)
						.level_count(1)
						.base_array_layer(0)
						.layer_count(1),
				);
			barriers.push(resolve_barrier);
		}

		if target.depth_image != vk::Image::null() {
			let depth_barrier = vk::ImageMemoryBarrier2::default()
				.src_stage_mask(vk::PipelineStageFlags2::EARLY_FRAGMENT_TESTS | vk::PipelineStageFlags2::LATE_FRAGMENT_TESTS)
				.src_access_mask(vk::AccessFlags2::NONE)
				.dst_stage_mask(vk::PipelineStageFlags2::EARLY_FRAGMENT_TESTS | vk::PipelineStageFlags2::LATE_FRAGMENT_TESTS)
				.dst_access_mask(vk::AccessFlags2::DEPTH_STENCIL_ATTACHMENT_WRITE)
				.old_layout(vk::ImageLayout::UNDEFINED)
				.new_layout(vk::ImageLayout::DEPTH_ATTACHMENT_OPTIMAL)
				.image(target.depth_image)
				.subresource_range(
					vk::ImageSubresourceRange::default()
						.aspect_mask(vk::ImageAspectFlags::DEPTH)
						.base_mip_level(0)
						.level_count(1)
						.base_array_layer(0)
						.layer_count(1),
				);
			barriers.push(depth_barrier);
		}

		let dep_info = vk::DependencyInfo::default()
			.dependency_flags(vk::DependencyFlags::BY_REGION)
			.image_memory_barriers(&barriers);

		unsafe {
			self.device.cmd_pipeline_barrier2(self.handle, &dep_info);
		}

		// 2. Begin Dynamic Rendering
		let clear_val = vk::ClearValue {
			color: vk::ClearColorValue {
				float32: clear_color.0,
			},
		};

		let has_resolve = target.resolve_view != vk::ImageView::null();
		let mut color_attachment = vk::RenderingAttachmentInfo::default()
			.image_view(target.color_view)
			.image_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
			.load_op(vk::AttachmentLoadOp::CLEAR)
			.store_op(if has_resolve {
				vk::AttachmentStoreOp::DONT_CARE
			} else {
				vk::AttachmentStoreOp::STORE
			})
			.clear_value(clear_val);

		if has_resolve {
			color_attachment = color_attachment
				.resolve_mode(vk::ResolveModeFlags::AVERAGE)
				.resolve_image_view(target.resolve_view)
				.resolve_image_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL);
		}

		let color_attachments = [color_attachment];

		let depth_attachment = vk::RenderingAttachmentInfo::default()
			.image_view(target.depth_view)
			.image_layout(vk::ImageLayout::DEPTH_ATTACHMENT_OPTIMAL)
			.load_op(vk::AttachmentLoadOp::CLEAR)
			.store_op(vk::AttachmentStoreOp::DONT_CARE)
			.clear_value(vk::ClearValue {
				depth_stencil: vk::ClearDepthStencilValue {
					depth: 1.0,
					stencil: 0,
				},
			});

		let render_area = vk::Rect2D {
			offset: vk::Offset2D { x: 0, y: 0 },
			extent: target.extent,
		};

		let mut rendering_info = vk::RenderingInfo::default()
			.render_area(render_area)
			.layer_count(1)
			.color_attachments(&color_attachments);

		if target.depth_image != vk::Image::null() {
			rendering_info = rendering_info.depth_attachment(&depth_attachment);
		}

		unsafe {
			self.device.cmd_begin_rendering(self.handle, &rendering_info);
		}
	}

	fn end_rendering(&mut self, target: &AshRenderTarget) {
		unsafe {
			self.device.cmd_end_rendering(self.handle);
		}

		let present_image = if target.resolve_image != vk::Image::null() {
			target.resolve_image
		} else {
			target.color_image
		};

		// Transition back to target.final_layout (PRESENT_SRC_KHR)
		let color_barrier = vk::ImageMemoryBarrier2::default()
			.src_stage_mask(vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT)
			.src_access_mask(vk::AccessFlags2::COLOR_ATTACHMENT_WRITE)
			.dst_stage_mask(vk::PipelineStageFlags2::BOTTOM_OF_PIPE)
			.dst_access_mask(vk::AccessFlags2::NONE)
			.old_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
			.new_layout(target.final_layout)
			.image(present_image)
			.subresource_range(
				vk::ImageSubresourceRange::default()
					.aspect_mask(vk::ImageAspectFlags::COLOR)
					.base_mip_level(0)
					.level_count(1)
					.base_array_layer(0)
					.layer_count(1),
			);

		let barriers = [color_barrier];
		let dep_info = vk::DependencyInfo::default().image_memory_barriers(&barriers);

		unsafe {
			self.device.cmd_pipeline_barrier2(self.handle, &dep_info);
		}
	}
}


pub struct CommandQueueInner {
	device: ash::Device,
	command_pool: vk::CommandPool,
	queue: vk::Queue,
	submit_lock: Mutex<()>,
}

unsafe impl Send for CommandQueueInner {}
unsafe impl Sync for CommandQueueInner {}

impl Drop for CommandQueueInner {
	fn drop(&mut self) {
		if self.command_pool != vk::CommandPool::null() {
			unsafe {
				self.device.destroy_command_pool(self.command_pool, None);
			}
			self.command_pool = vk::CommandPool::null();
		}
	}
}

#[derive(Clone)]
pub struct AshCommandQueue {
	inner: Arc<CommandQueueInner>,
}

unsafe impl Send for AshCommandQueue {}
unsafe impl Sync for AshCommandQueue {}

impl AshCommandQueue {
	pub fn new(device: &ash::Device, queue_family: u32, queue: vk::Queue) -> RhiResult<Self> {
		let pool_info = vk::CommandPoolCreateInfo::default()
			.flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
			.queue_family_index(queue_family);

		let command_pool = unsafe {
			device
				.create_command_pool(&pool_info, None)
				.map_err(|e| RhiError::BackendError(format!("Failed to create command pool: {e}")))?
		};

		Ok(Self {
			inner: Arc::new(CommandQueueInner {
				device: device.clone(),
				command_pool,
				queue,
				submit_lock: Mutex::new(()),
			}),
		})
	}
}

impl RhiCommandQueue<AshVulkanBackend> for AshCommandQueue {
	fn allocate_graphic_command(&self) -> RhiResult<AshCommandBuffer> {
		let alloc_info = vk::CommandBufferAllocateInfo::default()
			.command_pool(self.inner.command_pool)
			.level(vk::CommandBufferLevel::PRIMARY)
			.command_buffer_count(1);

		let bufs = unsafe {
			self.inner
				.device
				.allocate_command_buffers(&alloc_info)
				.map_err(|e| RhiError::BackendError(format!("Failed to allocate command buffer: {e}")))?
		};

		Ok(AshCommandBuffer::new(&self.inner.device, bufs[0]))
	}

	fn submit(&self, cmd: &AshCommandBuffer) -> RhiResult<()> {
		let cmds = [cmd.handle()];
		let submit_info = vk::SubmitInfo::default().command_buffers(&cmds);
		let submits = [submit_info];

		let _guard = self.inner.submit_lock.lock().unwrap();
		unsafe {
			self.inner
				.device
				.queue_submit(self.inner.queue, &submits, vk::Fence::null())
				.map_err(|e| RhiError::BackendError(format!("queue_submit failed: {e}")))?;
		}
		Ok(())
	}

	fn submit_with_sync(
		&self,
		cmd: &AshCommandBuffer,
		wait_semaphore: Option<&AshSemaphore>,
		signal_semaphore: Option<&AshSemaphore>,
		fence: Option<&AshFence>,
	) -> RhiResult<()> {
		let cmds = [cmd.handle()];

		let wait_sems = match wait_semaphore {
			Some(s) => vec![s.handle()],
			None => vec![],
		};
		let wait_stages = vec![vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT; wait_sems.len()];

		let sig_sems = match signal_semaphore {
			Some(s) => vec![s.handle()],
			None => vec![],
		};

		let fence_handle = fence.map(|f| f.handle()).unwrap_or(vk::Fence::null());

		let submit_info = vk::SubmitInfo::default()
			.command_buffers(&cmds)
			.wait_semaphores(&wait_sems)
			.wait_dst_stage_mask(&wait_stages)
			.signal_semaphores(&sig_sems);

		let submits = [submit_info];

		let _guard = self.inner.submit_lock.lock().unwrap();
		unsafe {
			self.inner
				.device
				.queue_submit(self.inner.queue, &submits, fence_handle)
				.map_err(|e| RhiError::BackendError(format!("queue_submit with sync failed: {e}")))?;
		}
		Ok(())
	}
}
