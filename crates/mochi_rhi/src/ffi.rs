/*
	This file is part of QAOS / Mochi

	This file is licensed under the GNU General Public License version 3 (GPL3).

	You should have received a copy of the GNU General Public License
	along with QAOS. If not, see <https://www.gnu.org/licenses/>.

	Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::ffi::c_char;


// Opaque types
#[repr(C)]
pub struct MochiDeviceOpaque { _private: [u8; 0] }
pub type MochiDeviceHandle = *mut MochiDeviceOpaque;

#[repr(C)]
pub struct MochiAllocatorOpaque { _private: [u8; 0] }
pub type MochiAllocatorHandle = *mut MochiAllocatorOpaque;

#[repr(C)]
pub struct MochiBufferOpaque { _private: [u8; 0] }
pub type MochiBufferHandle = *mut MochiBufferOpaque;

#[repr(C)]
pub struct MochiImage2Opaque { _private: [u8; 0] }
pub type MochiImage2Handle = *mut MochiImage2Opaque;

#[repr(C)]
pub struct MochiSampler2Opaque { _private: [u8; 0], }
pub type MochiSampler2Handle = *mut MochiSampler2Opaque;

#[repr(C)]
pub struct MochiSwapchainOpaque { _private: [u8; 0] }
pub type MochiSwapchainHandle = *mut MochiSwapchainOpaque;

#[repr(C)]
pub struct MochiSynchronizerOpaque { _private: [u8; 0] }
pub type MochiSynchronizerHandle = *mut MochiSynchronizerOpaque;

#[repr(C)]
pub struct MochiRenderTargetOpaque { _private: [u8; 0] }
pub type MochiRenderTargetHandle = *mut MochiRenderTargetOpaque;

#[repr(C)]
pub struct MochiCommandManagerOpaque { _private: [u8; 0] }
pub type MochiCommandManagerHandle = *mut MochiCommandManagerOpaque;

#[repr(C)]
pub struct MochiCommandOpaque { _private: [u8; 0] }
pub type MochiCommandHandle = *mut MochiCommandOpaque;

#[repr(C)]
pub struct MochiUploaderOpaque { _private: [u8; 0] }
pub type MochiUploaderHandle = *mut MochiUploaderOpaque;

#[repr(C)]
pub struct MochiShaderOpaque { _private: [u8; 0] }
pub type MochiShaderHandle = *mut MochiShaderOpaque;

#[repr(C)]
pub struct MochiPipelineMetaOpaque { _private: [u8; 0] }
pub type MochiPipelineMetaHandle = *mut MochiPipelineMetaOpaque;

#[repr(C)]
pub struct MochiPipelineOpaque { _private: [u8; 0] }
pub type MochiPipelineHandle = *mut MochiPipelineOpaque;


unsafe extern "C" {
	// Driver info
	pub unsafe fn mochi_rhi_get_driver_name() -> *const c_char;
	pub unsafe fn mochi_rhi_get_driver_version(out_version: *mut u32);

	// Device
	pub unsafe fn mochi_rhi_device_new(app_name: *const c_char, major: u16, minor: u16, patch: u16, build: u16) -> MochiDeviceHandle;
	pub unsafe fn mochi_rhi_device_dis(device: MochiDeviceHandle);

	pub unsafe fn mochi_rhi_device_wait_idle(device: MochiDeviceHandle);
	pub unsafe fn mochi_rhi_device_get_main_queue_family(device: MochiDeviceHandle) -> u32;

	pub unsafe fn mochi_rhi_device_init_descriptor_heap(device: MochiDeviceHandle, allocator: MochiAllocatorHandle);
	pub unsafe fn mochi_rhi_device_fini_descriptor_heap(device: MochiDeviceHandle);
	pub unsafe fn mochi_rhi_device_write_texture_descriptor(device: MochiDeviceHandle, view: MochiImage2Handle, sampler: MochiSampler2Handle) -> u32;
	pub unsafe fn mochi_rhi_device_get_descriptor_heap(device: MochiDeviceHandle) -> MochiBufferHandle;
	pub unsafe fn mochi_rhi_device_get_sampler_heap(device: MochiDeviceHandle) -> MochiBufferHandle;

	// Allocator
	pub unsafe fn mochi_rhi_allocator_new(device: MochiDeviceHandle) -> MochiAllocatorHandle;
	pub unsafe fn mochi_rhi_allocator_dis(allocator: MochiAllocatorHandle);

	pub unsafe fn mochi_rhi_allocator_alloc_buffer(allocator: MochiAllocatorHandle, size: u64, usage_flags: u32, create_flags: u32, location: u32) -> MochiBufferHandle;
	pub unsafe fn mochi_rhi_allocator_alloc_image2(allocator: MochiAllocatorHandle, width: u32, height: u32, format: u32, usage_flags: u32, tiling: u32, create_flags: u32, location: u32) -> MochiImage2Handle;
	pub unsafe fn mochi_rhi_allocator_alloc_sampler2(allocator: MochiAllocatorHandle, address_mode_u: u32, address_mode_v: u32, address_mode_w: u32, mag_filter: u32, min_filter: u32) -> MochiSampler2Handle;

	pub unsafe fn mochi_rhi_allocator_support_statistics(allocator: MochiAllocatorHandle) -> bool;
	pub unsafe fn mochi_rhi_allocator_stat_total_vram(allocator: MochiAllocatorHandle) -> u64;
	pub unsafe fn mochi_rhi_allocator_stat_cache_vram(allocator: MochiAllocatorHandle) -> u64;
	pub unsafe fn mochi_rhi_allocator_stat_used_vram(allocator: MochiAllocatorHandle) -> u64;
	pub unsafe fn mochi_rhi_allocator_stat_live_buffer(allocator: MochiAllocatorHandle) -> u64;
	pub unsafe fn mochi_rhi_allocator_stat_live_image2(allocator: MochiAllocatorHandle) -> u64;

	// Buffer
	pub unsafe fn mochi_rhi_buffer_dis(buffer: MochiBufferHandle);
	
	pub unsafe fn mochi_rhi_buffer_get_size(buffer: MochiBufferHandle) -> u64;
	pub unsafe fn mochi_rhi_buffer_map(buffer: MochiBufferHandle);
	pub unsafe fn mochi_rhi_buffer_unmap(buffer: MochiBufferHandle);
	pub unsafe fn mochi_rhi_buffer_flush(buffer: MochiBufferHandle, offset: u64, size: u64);
	pub unsafe fn mochi_rhi_buffer_get_mapped_ptr(buffer: MochiBufferHandle) -> *mut u8;
	pub unsafe fn mochi_rhi_buffer_get_device_address(buffer: MochiBufferHandle) -> u64;

	// Image2 & Sampler2
	pub unsafe fn mochi_rhi_image2_dis(image: MochiImage2Handle);
	pub unsafe fn mochi_rhi_sampler2_dis(sampler: MochiSampler2Handle);

	// Surface
	pub unsafe fn mochi_rhi_create_surface_wayland(device: MochiDeviceHandle, display: *mut std::ffi::c_void, surface: *mut std::ffi::c_void) -> u64;
	pub unsafe fn mochi_rhi_create_surface_xlib(device: MochiDeviceHandle, display: *mut std::ffi::c_void, window: std::ffi::c_ulong) -> u64;
	pub unsafe fn mochi_rhi_create_surface_xcb(device: MochiDeviceHandle, connection: *mut std::ffi::c_void, window: u32) -> u64;
	pub unsafe fn mochi_rhi_create_surface_win32(device: MochiDeviceHandle, hinstance: *mut std::ffi::c_void, hwnd: *mut std::ffi::c_void) -> u64;
	pub unsafe fn mochi_rhi_destroy_surface(device: MochiDeviceHandle, surface: u64);

	// Swapchain
	pub unsafe fn mochi_rhi_swapchain_new(device: MochiDeviceHandle, surface: u64, width: u32, height: u32) -> MochiSwapchainHandle;
	pub unsafe fn mochi_rhi_swapchain_dis(swapchain: MochiSwapchainHandle);

	pub unsafe fn mochi_rhi_swapchain_resize(swapchain: MochiSwapchainHandle, width: u32, height: u32);
	pub unsafe fn mochi_rhi_swapchain_acquire_next_image(swapchain: MochiSwapchainHandle, signal_semaphore: *mut std::ffi::c_void) -> u32;
	pub unsafe fn mochi_rhi_swapchain_present(swapchain: MochiSwapchainHandle, image_index: u32, wait_semaphore: *mut std::ffi::c_void);
	pub unsafe fn mochi_rhi_swapchain_get_image_count(swapchain: MochiSwapchainHandle) -> u32;
	pub unsafe fn mochi_rhi_swapchain_get_render_target(swapchain: MochiSwapchainHandle, index: u32) -> MochiRenderTargetHandle;
	pub unsafe fn mochi_rhi_swapchain_get_render_finished_semaphore(swapchain: MochiSwapchainHandle, index: u32) -> *mut std::ffi::c_void;

	// Synchronizer
	pub unsafe fn mochi_rhi_synchronizer_new(device: MochiDeviceHandle, max_frames_in_flight: u32) -> MochiSynchronizerHandle;
	pub unsafe fn mochi_rhi_synchronizer_dis(synchronizer: MochiSynchronizerHandle);
	pub unsafe fn mochi_rhi_synchronizer_begin_frame(synchronizer: MochiSynchronizerHandle);
	pub unsafe fn mochi_rhi_synchronizer_end_frame(synchronizer: MochiSynchronizerHandle);
	pub unsafe fn mochi_rhi_synchronizer_current_frame(synchronizer: MochiSynchronizerHandle) -> u32;
	pub unsafe fn mochi_rhi_synchronizer_get_image_available_semaphore(synchronizer: MochiSynchronizerHandle) -> *mut std::ffi::c_void;
	pub unsafe fn mochi_rhi_synchronizer_get_in_flight_fence(synchronizer: MochiSynchronizerHandle) -> *mut std::ffi::c_void;

	// RenderTarget
	pub unsafe fn mochi_rhi_render_target_get_color_format(target: MochiRenderTargetHandle) -> u32;
	pub unsafe fn mochi_rhi_render_target_get_depth_format(target: MochiRenderTargetHandle) -> u32;

	// Command Manager & Command
	pub unsafe fn mochi_rhi_command_manager_new(device: MochiDeviceHandle) -> MochiCommandManagerHandle;
	pub unsafe fn mochi_rhi_command_manager_dis(manager: MochiCommandManagerHandle);
	pub unsafe fn mochi_rhi_command_manager_allocate_graphics(manager: MochiCommandManagerHandle) -> MochiCommandHandle;
	pub unsafe fn mochi_rhi_command_manager_submit(manager: MochiCommandManagerHandle, cmd: MochiCommandHandle);
	pub unsafe fn mochi_rhi_command_manager_submit_with_sync(manager: MochiCommandManagerHandle, cmd: MochiCommandHandle, wait_semaphore: *mut std::ffi::c_void, signal_semaphore: *mut std::ffi::c_void, fence: *mut std::ffi::c_void);

	pub unsafe fn mochi_rhi_command_dis(cmd: MochiCommandHandle);
	pub unsafe fn mochi_rhi_command_begin(cmd: MochiCommandHandle);
	pub unsafe fn mochi_rhi_command_end(cmd: MochiCommandHandle);
	pub unsafe fn mochi_rhi_command_begin_rendering(cmd: MochiCommandHandle, target: MochiRenderTargetHandle, clear_r: f32, clear_g: f32, clear_b: f32, clear_a: f32);
	pub unsafe fn mochi_rhi_command_end_rendering(cmd: MochiCommandHandle, target: MochiRenderTargetHandle);
	pub unsafe fn mochi_rhi_command_set_viewport(cmd: MochiCommandHandle, x: f32, y: f32, width: f32, height: f32, min_depth: f32, max_depth: f32);
	pub unsafe fn mochi_rhi_command_set_scissor(cmd: MochiCommandHandle, x: i32, y: i32, width: u32, height: u32);
	pub unsafe fn mochi_rhi_command_bind_pipeline(cmd: MochiCommandHandle, pipeline: MochiPipelineHandle);
	pub unsafe fn mochi_rhi_command_bind_descriptor_heap(cmd: MochiCommandHandle, resource_heap: MochiBufferHandle, sampler_heap: MochiBufferHandle, resource_size: u64, sampler_size: u64);
	pub unsafe fn mochi_rhi_command_push_constant(cmd: MochiCommandHandle, stage_flags: u32, offset: u32, size: u32, data: *const std::ffi::c_void);
	pub unsafe fn mochi_rhi_command_draw(cmd: MochiCommandHandle, vertex_count: u32, instance_count: u32, first_vertex: u32, first_instance: u32);

	// Uploader
	pub unsafe fn mochi_rhi_uploader_new(device: MochiDeviceHandle) -> MochiUploaderHandle;
	pub unsafe fn mochi_rhi_uploader_dis(uploader: MochiUploaderHandle);
	pub unsafe fn mochi_rhi_uploader_copy_memory_to_buffer(uploader: MochiUploaderHandle, transfer_time: u32, src: *const std::ffi::c_void, dst: MochiBufferHandle);
	pub unsafe fn mochi_rhi_uploader_copy_memory_to_image(uploader: MochiUploaderHandle, transfer_time: u32, src: *const std::ffi::c_void, dst: MochiImage2Handle);

	// Shader
	pub unsafe fn mochi_rhi_shader_new(device: MochiDeviceHandle, stage: u32, code_ptr: *const u8, code_size: usize, entry: *const c_char) -> MochiShaderHandle;
	pub unsafe fn mochi_rhi_shader_dis(shader: MochiShaderHandle);

	// Pipeline
	pub unsafe fn mochi_rhi_pipeline_meta_new(push_stage_flags: u32, push_size: u32) -> MochiPipelineMetaHandle;
	pub unsafe fn mochi_rhi_pipeline_meta_dis(meta: MochiPipelineMetaHandle);
	pub unsafe fn mochi_rhi_pipeline_graphics_new( device: MochiDeviceHandle, meta: MochiPipelineMetaHandle, shaders: *const MochiShaderHandle, shader_count: u32, polygon_mode: u8, primitive_topology: u8, color_format: u32, depth_format: u32) -> MochiPipelineHandle;
	pub unsafe fn mochi_rhi_pipeline_dis(pipeline: MochiPipelineHandle);
}
