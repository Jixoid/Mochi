/*
	This file is part of QAOS

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::ffi::CString;
use raw_window_handle::{HasDisplayHandle, HasWindowHandle, RawDisplayHandle, RawWindowHandle};
use mochi_math::Extent;

use crate::error::{RhiError, RhiResult};
use crate::ffi::*;
use crate::sync::{Fence as CppFence, Semaphore as CppSemaphore};
use crate::traits::*;
use crate::types::*;


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CppVulkanBackend;

impl RhiBackend for CppVulkanBackend {
	type Device = CppDevice;
	type Allocator = CppAllocator;
	type Buffer = CppBuffer;
	type Image = CppImage;
	type ImageView = CppImageView;
	type Sampler = CppSampler;
	type Shader = CppShader;
	type Pipeline = CppPipeline;
	type PipelineMeta = CppPipelineMeta;
	type CommandBuffer = CppCommandBuffer;
	type CommandQueue = CppCommandQueue;
	type Swapchain = CppSwapchain;
	type RenderTarget = CppRenderTarget;
	type Semaphore = CppSemaphore;
	type Fence = CppFence;
}

// Device
pub struct CppDevice {
	pub handle: MochiDeviceHandle,
	pub main_queue_family: u32,
}
unsafe impl Send for CppDevice {}
unsafe impl Sync for CppDevice {}

impl CppDevice {
	pub fn new(app_name: &str, major: u16, minor: u16, patch: u16, build: u16) -> RhiResult<Self> {
		let c_name = CString::new(app_name).map_err(|e| RhiError::InitializationFailed(e.to_string()))?;
		let handle = unsafe { mochi_rhi_device_new(c_name.as_ptr(), major, minor, patch, build) };
		if handle.is_null() {
			return Err(RhiError::InitializationFailed("Failed to create C++ Vulkan Device".into()));
		}
		let main_queue_family = unsafe { mochi_rhi_device_get_main_queue_family(handle) };
		Ok(Self {
			handle,
			main_queue_family,
		})
	}
}

impl RhiDevice<CppVulkanBackend> for CppDevice {
	fn wait_idle(&self) -> RhiResult<()> {
		unsafe { mochi_rhi_device_wait_idle(self.handle); }
		Ok(())
	}

	fn main_queue_family(&self) -> u32 {
		self.main_queue_family
	}

	fn create_semaphore(&self) -> RhiResult<CppSemaphore> {
		Ok(CppSemaphore(std::ptr::null_mut()))
	}

	fn create_fence(&self, _signaled: bool) -> RhiResult<CppFence> {
		Ok(CppFence(std::ptr::null_mut()))
	}

	fn wait_fence(&self, _fence: &CppFence, _timeout_ns: u64) -> RhiResult<()> {
		Ok(())
	}

	fn reset_fence(&self, _fence: &CppFence) -> RhiResult<()> {
		Ok(())
	}

	fn create_surface<W: HasWindowHandle + HasDisplayHandle>(&self, window: &W) -> RhiResult<u64> {
		let display_handle = window
			.display_handle()
			.map_err(|e| RhiError::SurfaceCreation(e.to_string()))?;

		let window_handle = window
			.window_handle()
			.map_err(|e| RhiError::SurfaceCreation(e.to_string()))?;

		let surface_handle = match (display_handle.as_raw(), window_handle.as_raw()) {
			(RawDisplayHandle::Wayland(d), RawWindowHandle::Wayland(w)) => unsafe {
				mochi_rhi_create_surface_wayland(
					self.handle,
					d.display.as_ptr(),
					w.surface.as_ptr(),
				)
			},
			(RawDisplayHandle::Xlib(d), RawWindowHandle::Xlib(w)) => {
				let dpy = d.display.map(|p| p.as_ptr()).unwrap_or(std::ptr::null_mut());
				unsafe { mochi_rhi_create_surface_xlib(self.handle, dpy, w.window) }
			}
			(RawDisplayHandle::Xcb(d), RawWindowHandle::Xcb(w)) => {
				let conn = d.connection.map(|p| p.as_ptr()).unwrap_or(std::ptr::null_mut());
				unsafe { mochi_rhi_create_surface_xcb(self.handle, conn, w.window.get()) }
			}
			(RawDisplayHandle::Windows(_), RawWindowHandle::Win32(w)) => {
				let hinstance = w
					.hinstance
					.map(|h| h.get() as *mut std::ffi::c_void)
					.unwrap_or(std::ptr::null_mut());
				let hwnd = w.hwnd.get() as *mut std::ffi::c_void;
				unsafe { mochi_rhi_create_surface_win32(self.handle, hinstance, hwnd) }
			}
			(disp, win) => {
				return Err(RhiError::SurfaceCreation(format!(
					"unsupported window/display handle combination: {:?}, {:?}",
					disp, win
				)));
			}
		};

		if surface_handle == 0 {
			return Err(RhiError::SurfaceCreation("failed to create surface".to_string()));
		}

		Ok(surface_handle)
	}

	fn destroy_surface(&self, surface_handle: u64) {
		if surface_handle != 0 && !self.handle.is_null() {
			unsafe { mochi_rhi_destroy_surface(self.handle, surface_handle); }
		}
	}

	fn init_descriptor_heap(&self, allocator: &CppAllocator) -> RhiResult<()> {
		unsafe { mochi_rhi_device_init_descriptor_heap(self.handle, allocator.handle); }
		Ok(())
	}

	fn fini_descriptor_heap(&self) {
		unsafe { mochi_rhi_device_fini_descriptor_heap(self.handle); }
	}

	fn write_texture_descriptor(&self, view: &CppImageView, sampler: &CppSampler) -> u32 {
		unsafe { mochi_rhi_device_write_texture_descriptor(self.handle, view.handle, sampler.handle) }
	}

	fn descriptor_heap(&self) -> Option<CppBuffer> {
		let h = unsafe { mochi_rhi_device_get_descriptor_heap(self.handle) };
		if h.is_null() { None } else { Some(CppBuffer { handle: h, owned: false }) }
	}

	fn sampler_heap(&self) -> Option<CppBuffer> {
		let h = unsafe { mochi_rhi_device_get_sampler_heap(self.handle) };
		if h.is_null() { None } else { Some(CppBuffer { handle: h, owned: false }) }
	}
}

impl Drop for CppDevice {
	fn drop(&mut self) {
		if !self.handle.is_null() {
			unsafe {
				mochi_rhi_device_wait_idle(self.handle);
				mochi_rhi_device_dis(self.handle);
			}
			self.handle = std::ptr::null_mut();
		}
	}
}

// Allocator
pub struct CppAllocator {
	pub handle: MochiAllocatorHandle,
}
unsafe impl Send for CppAllocator {}
unsafe impl Sync for CppAllocator {}

impl RhiAllocator<CppVulkanBackend> for CppAllocator {
	fn alloc_buffer(
		&self,
		size: u64,
		usage: BufferUsage,
		create: AllocationCreate,
		location: AllocationLocation,
	) -> RhiResult<CppBuffer> {
		let handle = unsafe {
			mochi_rhi_allocator_alloc_buffer(
				self.handle,
				size,
				usage.bits(),
				create.bits(),
				location as u32,
			)
		};
		if handle.is_null() {
			Err(RhiError::OutOfMemory)
		} else {
			Ok(CppBuffer { handle, owned: true })
		}
	}

	fn alloc_image(
		&self,
		ext: Extent<2, u32>,
		format: Format,
		usage: ImageUsage,
		tiling: ImageTiling,
		create: AllocationCreate,
		location: AllocationLocation,
	) -> RhiResult<CppImage> {
		let handle = unsafe {
			mochi_rhi_allocator_alloc_image2(
				self.handle,
				ext[0],
				ext[1],
				format as u32,
				usage.bits(),
				tiling as u32,
				create.bits(),
				location as u32,
			)
		};
		if handle.is_null() {
			Err(RhiError::OutOfMemory)
		} else {
			Ok(CppImage { handle, ext, format })
		}
	}

	fn alloc_sampler(
		&self,
		address_mode: Extent<3, SamplerAddressMode>,
		mag_filter: SamplerFilter,
		min_filter: SamplerFilter,
	) -> RhiResult<CppSampler> {
		let handle = unsafe {
			mochi_rhi_allocator_alloc_sampler2(
				self.handle,
				address_mode[0] as u32,
				address_mode[1] as u32,
				address_mode[2] as u32,
				mag_filter as u32,
				min_filter as u32,
			)
		};
		if handle.is_null() {
			Err(RhiError::ResourceCreation("Failed to allocate Sampler".into()))
		} else {
			Ok(CppSampler { handle })
		}
	}
}

impl Drop for CppAllocator {
	fn drop(&mut self) {
		if !self.handle.is_null() {
			unsafe { mochi_rhi_allocator_dis(self.handle); }
			self.handle = std::ptr::null_mut();
		}
	}
}

// Buffer
pub struct CppBuffer {
	pub handle: MochiBufferHandle,
	pub owned: bool,
}
unsafe impl Send for CppBuffer {}
unsafe impl Sync for CppBuffer {}

impl RhiBuffer for CppBuffer {
	fn size(&self) -> u64 {
		unsafe { mochi_rhi_buffer_get_size(self.handle) }
	}

	fn device_address(&self) -> u64 {
		unsafe { mochi_rhi_buffer_get_device_address(self.handle) }
	}

	fn map(&mut self) -> RhiResult<*mut u8> {
		unsafe {
			mochi_rhi_buffer_map(self.handle);
			let ptr = mochi_rhi_buffer_get_mapped_ptr(self.handle);
			if ptr.is_null() {
				Err(RhiError::BackendError("Failed to map buffer".into()))
			} else {
				Ok(ptr)
			}
		}
	}

	fn unmap(&mut self) {
		unsafe { mochi_rhi_buffer_unmap(self.handle); }
	}

	fn flush(&mut self, offset: u64, size: u64) -> RhiResult<()> {
		unsafe { mochi_rhi_buffer_flush(self.handle, offset, size); }
		Ok(())
	}
}

impl Drop for CppBuffer {
	fn drop(&mut self) {
		if self.owned && !self.handle.is_null() {
			unsafe { mochi_rhi_buffer_dis(self.handle); }
			self.handle = std::ptr::null_mut();
		}
	}
}

// Image & ImageView
pub struct CppImage {
	pub handle: MochiImage2Handle,
	pub ext: Extent<2, u32>,
	pub format: Format,
}
unsafe impl Send for CppImage {}
unsafe impl Sync for CppImage {}

impl RhiImage for CppImage {
	fn ext(&self) -> Extent<2, u32> {
		self.ext
	}

	fn format(&self) -> Format {
		self.format
	}
}

impl Drop for CppImage {
	fn drop(&mut self) {
		if !self.handle.is_null() {
			unsafe { mochi_rhi_image2_dis(self.handle); }
			self.handle = std::ptr::null_mut();
		}
	}
}

pub struct CppImageView {
	pub handle: MochiImage2Handle,
}
unsafe impl Send for CppImageView {}
unsafe impl Sync for CppImageView {}

impl RhiImageView for CppImageView {}

// Sampler
pub struct CppSampler {
	pub handle: MochiSampler2Handle,
}
unsafe impl Send for CppSampler {}
unsafe impl Sync for CppSampler {}

impl RhiSampler for CppSampler {}

impl Drop for CppSampler {
	fn drop(&mut self) {
		if !self.handle.is_null() {
			unsafe { mochi_rhi_sampler2_dis(self.handle); }
			self.handle = std::ptr::null_mut();
		}
	}
}

// Shader
pub struct CppShader {
	pub handle: MochiShaderHandle,
	pub stage: ShaderStage,
	pub entry: String,
}
unsafe impl Send for CppShader {}
unsafe impl Sync for CppShader {}

impl RhiShader for CppShader {
	fn stage(&self) -> ShaderStage {
		self.stage
	}

	fn entry(&self) -> &str {
		&self.entry
	}
}

impl Drop for CppShader {
	fn drop(&mut self) {
		if !self.handle.is_null() {
			unsafe { mochi_rhi_shader_dis(self.handle); }
			self.handle = std::ptr::null_mut();
		}
	}
}

// Pipeline Meta & Pipeline
pub struct CppPipelineMeta {
	pub handle: MochiPipelineMetaHandle,
}
unsafe impl Send for CppPipelineMeta {}
unsafe impl Sync for CppPipelineMeta {}

impl RhiPipelineMeta for CppPipelineMeta {}

impl Drop for CppPipelineMeta {
	fn drop(&mut self) {
		if !self.handle.is_null() {
			unsafe { mochi_rhi_pipeline_meta_dis(self.handle); }
			self.handle = std::ptr::null_mut();
		}
	}
}

pub struct CppPipeline {
	pub handle: MochiPipelineHandle,
	pub kind: PipelineKind,
}
unsafe impl Send for CppPipeline {}
unsafe impl Sync for CppPipeline {}

impl RhiPipeline for CppPipeline {
	fn kind(&self) -> PipelineKind {
		self.kind
	}
}

impl Drop for CppPipeline {
	fn drop(&mut self) {
		if !self.handle.is_null() {
			unsafe { mochi_rhi_pipeline_dis(self.handle); }
			self.handle = std::ptr::null_mut();
		}
	}
}

// RenderTarget
#[derive(Clone, Copy)]
pub struct CppRenderTarget {
	pub handle: MochiRenderTargetHandle,
}
unsafe impl Send for CppRenderTarget {}
unsafe impl Sync for CppRenderTarget {}

impl RhiRenderTarget for CppRenderTarget {
	fn color_format(&self) -> Format {
		let raw = unsafe { mochi_rhi_render_target_get_color_format(self.handle) };
		Format::from_raw(raw)
	}

	fn depth_format(&self) -> Format {
		let raw = unsafe { mochi_rhi_render_target_get_depth_format(self.handle) };
		Format::from_raw(raw)
	}
}

// CommandBuffer
pub struct CppCommandBuffer {
	pub handle: MochiCommandHandle,
}
unsafe impl Send for CppCommandBuffer {}
unsafe impl Sync for CppCommandBuffer {}

impl RhiCommandBuffer<CppVulkanBackend> for CppCommandBuffer {
	fn begin(&mut self) -> RhiResult<()> {
		unsafe { mochi_rhi_command_begin(self.handle); }
		Ok(())
	}

	fn end(&mut self) -> RhiResult<()> {
		unsafe { mochi_rhi_command_end(self.handle); }
		Ok(())
	}

	fn set_viewport(&mut self, vp: Viewport) {
		unsafe {
			mochi_rhi_command_set_viewport(
				self.handle,
				vp.x,
				vp.y,
				vp.width,
				vp.height,
				vp.min_depth,
				vp.max_depth,
			);
		}
	}

	fn set_scissor(&mut self, s: Rect2D) {
		unsafe {
			mochi_rhi_command_set_scissor(self.handle, s.x, s.y, s.width, s.height);
		}
	}

	fn bind_pipeline(&mut self, pipeline: &CppPipeline) {
		unsafe { mochi_rhi_command_bind_pipeline(self.handle, pipeline.handle); }
	}

	fn bind_descriptor_heap(
		&mut self,
		res_heap: Option<&CppBuffer>,
		samp_heap: Option<&CppBuffer>,
		res_sz: u64,
		samp_sz: u64,
	) {
		let r_ptr = res_heap.map(|b| b.handle).unwrap_or(std::ptr::null_mut());
		let s_ptr = samp_heap.map(|b| b.handle).unwrap_or(std::ptr::null_mut());
		unsafe {
			mochi_rhi_command_bind_descriptor_heap(self.handle, r_ptr, s_ptr, res_sz, samp_sz);
		}
	}

	fn push_constant(&mut self, stages: ShaderStage, offset: u32, bytes: &[u8]) {
		unsafe {
			mochi_rhi_command_push_constant(
				self.handle,
				stages.bits(),
				offset,
				bytes.len() as u32,
				bytes.as_ptr() as *const std::ffi::c_void,
			);
		}
	}

	fn draw(&mut self, vertex_count: u32, instance_count: u32, first_vertex: u32, first_instance: u32) {
		unsafe {
			mochi_rhi_command_draw(self.handle, vertex_count, instance_count, first_vertex, first_instance);
		}
	}

	fn begin_rendering(&mut self, target: &CppRenderTarget, clear_color: ClearColor) {
		unsafe {
			mochi_rhi_command_begin_rendering(
				self.handle,
				target.handle,
				clear_color.0[0],
				clear_color.0[1],
				clear_color.0[2],
				clear_color.0[3],
			);
		}
	}

	fn end_rendering(&mut self, target: &CppRenderTarget) {
		unsafe {
			mochi_rhi_command_end_rendering(self.handle, target.handle);
		}
	}
}

impl Drop for CppCommandBuffer {
	fn drop(&mut self) {
		if !self.handle.is_null() {
			unsafe { mochi_rhi_command_dis(self.handle); }
			self.handle = std::ptr::null_mut();
		}
	}
}

// CommandQueue
pub struct CppCommandQueue {
	pub handle: MochiCommandManagerHandle,
}
unsafe impl Send for CppCommandQueue {}
unsafe impl Sync for CppCommandQueue {}

impl RhiCommandQueue<CppVulkanBackend> for CppCommandQueue {
	fn allocate_graphic_command(&self) -> RhiResult<CppCommandBuffer> {
		let cmd = unsafe { mochi_rhi_command_manager_allocate_graphics(self.handle) };
		if cmd.is_null() {
			Err(RhiError::BackendError("Failed to allocate graphic command buffer".into()))
		} else {
			Ok(CppCommandBuffer { handle: cmd })
		}
	}

	fn submit(&self, cmd: &CppCommandBuffer) -> RhiResult<()> {
		unsafe { mochi_rhi_command_manager_submit(self.handle, cmd.handle); }
		Ok(())
	}

	fn submit_with_sync(
		&self,
		cmd: &CppCommandBuffer,
		wait_semaphore: Option<&CppSemaphore>,
		signal_semaphore: Option<&CppSemaphore>,
		fence: Option<&CppFence>,
	) -> RhiResult<()> {
		let wait = wait_semaphore.map(|s| s.0).unwrap_or(std::ptr::null_mut());
		let sig = signal_semaphore.map(|s| s.0).unwrap_or(std::ptr::null_mut());
		let fen = fence.map(|f| f.0).unwrap_or(std::ptr::null_mut());
		unsafe {
			mochi_rhi_command_manager_submit_with_sync(self.handle, cmd.handle, wait, sig, fen);
		}
		Ok(())
	}
}

impl Drop for CppCommandQueue {
	fn drop(&mut self) {
		if !self.handle.is_null() {
			unsafe { mochi_rhi_command_manager_dis(self.handle); }
			self.handle = std::ptr::null_mut();
		}
	}
}

// Swapchain
pub struct CppSwapchain {
	pub handle: MochiSwapchainHandle,
	pub targets: Vec<CppRenderTarget>,
	pub render_finished_semaphores: Vec<CppSemaphore>,
}
unsafe impl Send for CppSwapchain {}
unsafe impl Sync for CppSwapchain {}

impl CppSwapchain {
	pub fn new(device: &CppDevice, surface: u64, width: u32, height: u32) -> RhiResult<Self> {
		let handle = unsafe { mochi_rhi_swapchain_new(device.handle, surface, width, height) };
		if handle.is_null() {
			return Err(RhiError::ResourceCreation("Failed to create Swapchain".into()));
		}
		let count = unsafe { mochi_rhi_swapchain_get_image_count(handle) };
		let mut targets = Vec::with_capacity(count as usize);
		let mut render_finished_semaphores = Vec::with_capacity(count as usize);

		for i in 0..count {
			let t = unsafe { mochi_rhi_swapchain_get_render_target(handle, i) };
			targets.push(CppRenderTarget { handle: t });
			let sem = unsafe { mochi_rhi_swapchain_get_render_finished_semaphore(handle, i) };
			render_finished_semaphores.push(CppSemaphore(sem));
		}

		Ok(Self {
			handle,
			targets,
			render_finished_semaphores,
		})
	}
}

impl RhiSwapchain<CppVulkanBackend> for CppSwapchain {
	fn resize(&mut self, width: u32, height: u32) -> RhiResult<()> {
		if !self.handle.is_null() {
			unsafe { mochi_rhi_swapchain_resize(self.handle, width, height); }
			let count = unsafe { mochi_rhi_swapchain_get_image_count(self.handle) };
			self.targets.clear();
			self.render_finished_semaphores.clear();
			for i in 0..count {
				let t = unsafe { mochi_rhi_swapchain_get_render_target(self.handle, i) };
				self.targets.push(CppRenderTarget { handle: t });
				let sem = unsafe { mochi_rhi_swapchain_get_render_finished_semaphore(self.handle, i) };
				self.render_finished_semaphores.push(CppSemaphore(sem));
			}
		}
		Ok(())
	}

	fn acquire_next_image(&mut self, signal_sem: Option<&CppSemaphore>) -> RhiResult<u32> {
		let sem = signal_sem.map(|s| s.0).unwrap_or(std::ptr::null_mut());
		let idx = unsafe { mochi_rhi_swapchain_acquire_next_image(self.handle, sem) };
		Ok(idx)
	}

	fn present(&mut self, image_index: u32, wait_sem: Option<&CppSemaphore>) -> RhiResult<()> {
		let sem = wait_sem.map(|s| s.0).unwrap_or(std::ptr::null_mut());
		unsafe { mochi_rhi_swapchain_present(self.handle, image_index, sem); }
		Ok(())
	}

	fn image_count(&self) -> u32 {
		self.targets.len() as u32
	}

	fn render_target(&self, index: u32) -> Option<&CppRenderTarget> {
		self.targets.get(index as usize)
	}

	fn render_finished_semaphore(&self, index: u32) -> &CppSemaphore {
		&self.render_finished_semaphores[index as usize]
	}
}

impl Drop for CppSwapchain {
	fn drop(&mut self) {
		if !self.handle.is_null() {
			unsafe { mochi_rhi_swapchain_dis(self.handle); }
			self.handle = std::ptr::null_mut();
		}
	}
}
