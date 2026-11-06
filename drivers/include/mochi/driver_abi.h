/*
  This file is part of QAOS / Mochi

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/

#pragma once

#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>

#ifdef __cplusplus
extern "C" {
#endif

// Opaque Handles for RHI & AHI
typedef struct MochiDeviceOpaque*          MochiDeviceHandle;
typedef struct MochiAllocatorOpaque*       MochiAllocatorHandle;
typedef struct MochiBufferOpaque*          MochiBufferHandle;
typedef struct MochiImage2Opaque*          MochiImage2Handle;
typedef struct MochiSampler2Opaque*        MochiSampler2Handle;
typedef struct MochiAudioDeviceOpaque*     MochiAudioDeviceHandle;
typedef struct MochiSynchronizerOpaque*    MochiSynchronizerHandle;
typedef struct MochiRenderTargetOpaque*    MochiRenderTargetHandle;
typedef struct MochiCommandManagerOpaque*  MochiCommandManagerHandle;
typedef struct MochiCommandOpaque*         MochiCommandHandle;
typedef struct MochiUploaderOpaque*        MochiUploaderHandle;
typedef struct MochiShaderOpaque*          MochiShaderHandle;
typedef struct MochiPipelineMetaOpaque*    MochiPipelineMetaHandle;
typedef struct MochiPipelineOpaque*        MochiPipelineHandle;

// Driver Info
const char* mochi_rhi_get_driver_name(void);
void        mochi_rhi_get_driver_version(uint32_t out_version[4]);

// Device Management
MochiDeviceHandle mochi_rhi_device_create(const char* app_name, uint16_t major, uint16_t minor, uint16_t patch, uint16_t build);
void              mochi_rhi_device_destroy(MochiDeviceHandle device);
MochiDeviceHandle mochi_rhi_device_new(const char* app_name, uint16_t major, uint16_t minor, uint16_t patch, uint16_t build);
void              mochi_rhi_device_dis(MochiDeviceHandle device);
void              mochi_rhi_device_wait_idle(MochiDeviceHandle device);

// Queues
uint32_t          mochi_rhi_device_get_main_queue_family(MochiDeviceHandle device);

// Descriptor Heap
void              mochi_rhi_device_init_descriptor_heap(MochiDeviceHandle device, MochiAllocatorHandle allocator);
void              mochi_rhi_device_fini_descriptor_heap(MochiDeviceHandle device);
uint32_t          mochi_rhi_device_write_texture_descriptor(MochiDeviceHandle device, MochiImage2Handle view, MochiSampler2Handle sampler);
MochiBufferHandle mochi_rhi_device_get_descriptor_heap(MochiDeviceHandle device);
MochiBufferHandle mochi_rhi_device_get_sampler_heap(MochiDeviceHandle device);

// Allocator Management
MochiAllocatorHandle mochi_rhi_allocator_create(MochiDeviceHandle device);
void                 mochi_rhi_allocator_destroy(MochiAllocatorHandle allocator);
MochiAllocatorHandle mochi_rhi_allocator_new(MochiDeviceHandle device);
void                 mochi_rhi_allocator_dis(MochiAllocatorHandle allocator);

MochiBufferHandle    mochi_rhi_allocator_alloc_buffer(
  MochiAllocatorHandle allocator,
  uint64_t size,
  uint32_t usage_flags,
  uint32_t create_flags,
  uint32_t location
);

MochiImage2Handle    mochi_rhi_allocator_alloc_image2(
  MochiAllocatorHandle allocator,
  uint32_t width,
  uint32_t height,
  uint32_t format,
  uint32_t usage_flags,
  uint32_t tiling,
  uint32_t create_flags,
  uint32_t location
);

MochiSampler2Handle  mochi_rhi_allocator_alloc_sampler2(
  MochiAllocatorHandle allocator,
  uint32_t address_mode_u,
  uint32_t address_mode_v,
  uint32_t address_mode_w,
  uint32_t mag_filter,
  uint32_t min_filter
);

// Statistics
bool     mochi_rhi_allocator_support_statistics(MochiAllocatorHandle allocator);
uint64_t mochi_rhi_allocator_stat_total_vram(MochiAllocatorHandle allocator);
uint64_t mochi_rhi_allocator_stat_cache_vram(MochiAllocatorHandle allocator);
uint64_t mochi_rhi_allocator_stat_used_vram(MochiAllocatorHandle allocator);
uint64_t mochi_rhi_allocator_stat_live_buffer(MochiAllocatorHandle allocator);
uint64_t mochi_rhi_allocator_stat_live_image2(MochiAllocatorHandle allocator);

// Buffer Operations
void     mochi_rhi_buffer_destroy(MochiBufferHandle buffer);
void     mochi_rhi_buffer_dis(MochiBufferHandle buffer);
uint64_t mochi_rhi_buffer_get_size(MochiBufferHandle buffer);
void     mochi_rhi_buffer_map(MochiBufferHandle buffer);
void     mochi_rhi_buffer_unmap(MochiBufferHandle buffer);
void     mochi_rhi_buffer_flush(MochiBufferHandle buffer, uint64_t offset, uint64_t size);
void*    mochi_rhi_buffer_get_mapped_ptr(MochiBufferHandle buffer);
uint64_t mochi_rhi_buffer_get_device_address(MochiBufferHandle buffer);

// Surface & Swapchain Management
typedef struct MochiSwapchainOpaque* MochiSwapchainHandle;

uint64_t             mochi_rhi_create_surface_wayland(MochiDeviceHandle device, void* display, void* surface);
uint64_t             mochi_rhi_create_surface_xlib(MochiDeviceHandle device, void* display, unsigned long window);
uint64_t             mochi_rhi_create_surface_xcb(MochiDeviceHandle device, void* connection, uint32_t window);
uint64_t             mochi_rhi_create_surface_win32(MochiDeviceHandle device, void* hinstance, void* hwnd);
void                 mochi_rhi_destroy_surface(MochiDeviceHandle device, uint64_t surface);

MochiSwapchainHandle mochi_rhi_swapchain_create(MochiDeviceHandle device, uint64_t surface, uint32_t width, uint32_t height);
void                 mochi_rhi_swapchain_destroy(MochiSwapchainHandle swapchain);
MochiSwapchainHandle mochi_rhi_swapchain_new(MochiDeviceHandle device, uint64_t surface, uint32_t width, uint32_t height);
void                 mochi_rhi_swapchain_dis(MochiSwapchainHandle swapchain);
void                 mochi_rhi_swapchain_resize(MochiSwapchainHandle swapchain, uint32_t width, uint32_t height);
uint32_t             mochi_rhi_swapchain_acquire_next_image(MochiSwapchainHandle swapchain, void* signal_semaphore);
void                 mochi_rhi_swapchain_present(MochiSwapchainHandle swapchain, uint32_t image_index, void* wait_semaphore);
uint32_t             mochi_rhi_swapchain_get_image_count(MochiSwapchainHandle swapchain);

// Image2 & Sampler2
void mochi_rhi_image2_destroy(MochiImage2Handle image);
void mochi_rhi_image2_dis(MochiImage2Handle image);
void mochi_rhi_sampler2_destroy(MochiSampler2Handle sampler);
void mochi_rhi_sampler2_dis(MochiSampler2Handle sampler);

// Synchronizer Management
MochiSynchronizerHandle mochi_rhi_synchronizer_create(MochiDeviceHandle device, uint32_t max_frames_in_flight);
void                    mochi_rhi_synchronizer_destroy(MochiSynchronizerHandle synchronizer);
MochiSynchronizerHandle mochi_rhi_synchronizer_new(MochiDeviceHandle device, uint32_t max_frames_in_flight);
void                    mochi_rhi_synchronizer_dis(MochiSynchronizerHandle synchronizer);
void                    mochi_rhi_synchronizer_begin_frame(MochiSynchronizerHandle synchronizer);
void                    mochi_rhi_synchronizer_end_frame(MochiSynchronizerHandle synchronizer);
uint32_t                mochi_rhi_synchronizer_current_frame(MochiSynchronizerHandle synchronizer);
void*                   mochi_rhi_synchronizer_get_image_available_semaphore(MochiSynchronizerHandle synchronizer);
void*                   mochi_rhi_synchronizer_get_in_flight_fence(MochiSynchronizerHandle synchronizer);

// RenderTarget & Swapchain RenderTarget
MochiRenderTargetHandle mochi_rhi_swapchain_get_render_target(MochiSwapchainHandle swapchain, uint32_t index);
void*                   mochi_rhi_swapchain_get_render_finished_semaphore(MochiSwapchainHandle swapchain, uint32_t index);
uint32_t                mochi_rhi_render_target_get_color_format(MochiRenderTargetHandle target);
uint32_t                mochi_rhi_render_target_get_depth_format(MochiRenderTargetHandle target);

// Command Manager & Command
MochiCommandManagerHandle mochi_rhi_command_manager_create(MochiDeviceHandle device);
void                      mochi_rhi_command_manager_destroy(MochiCommandManagerHandle manager);
MochiCommandManagerHandle mochi_rhi_command_manager_new(MochiDeviceHandle device);
void                      mochi_rhi_command_manager_dis(MochiCommandManagerHandle manager);
MochiCommandHandle        mochi_rhi_command_manager_allocate_graphics(MochiCommandManagerHandle manager);
void                      mochi_rhi_command_manager_submit(MochiCommandManagerHandle manager, MochiCommandHandle cmd);
void                      mochi_rhi_command_manager_submit_with_sync(
  MochiCommandManagerHandle manager,
  MochiCommandHandle cmd,
  void* wait_semaphore,
  void* signal_semaphore,
  void* fence
);

void mochi_rhi_command_destroy(MochiCommandHandle cmd);
void mochi_rhi_command_dis(MochiCommandHandle cmd);
void mochi_rhi_command_begin(MochiCommandHandle cmd);
void mochi_rhi_command_end(MochiCommandHandle cmd);
void mochi_rhi_command_begin_rendering(
  MochiCommandHandle cmd,
  MochiRenderTargetHandle target,
  float clear_r, float clear_g, float clear_b, float clear_a
);
void mochi_rhi_command_end_rendering(MochiCommandHandle cmd, MochiRenderTargetHandle target);
void mochi_rhi_command_set_viewport(
  MochiCommandHandle cmd,
  float x, float y, float width, float height, float min_depth, float max_depth
);
void mochi_rhi_command_set_scissor(
  MochiCommandHandle cmd,
  int32_t x, int32_t y, uint32_t width, uint32_t height
);
void mochi_rhi_command_bind_pipeline(MochiCommandHandle cmd, MochiPipelineHandle pipeline);
void mochi_rhi_command_bind_descriptor_heap(
  MochiCommandHandle cmd,
  MochiBufferHandle resource_heap,
  MochiBufferHandle sampler_heap,
  uint64_t resource_size,
  uint64_t sampler_size
);
void mochi_rhi_command_push_constant(
  MochiCommandHandle cmd,
  uint32_t stage_flags,
  uint32_t offset,
  uint32_t size,
  const void* data
);
void mochi_rhi_command_draw(
  MochiCommandHandle cmd,
  uint32_t vertex_count,
  uint32_t instance_count,
  uint32_t first_vertex,
  uint32_t first_instance
);

// Uploader Management
MochiUploaderHandle mochi_rhi_uploader_new(MochiDeviceHandle device);
void                mochi_rhi_uploader_dis(MochiUploaderHandle uploader);
void                mochi_rhi_uploader_copy_memory_to_buffer(
  MochiUploaderHandle uploader,
  uint32_t transfer_time,
  const void* src,
  MochiBufferHandle dst
);
void                mochi_rhi_uploader_copy_memory_to_image(
  MochiUploaderHandle uploader,
  uint32_t transfer_time,
  const void* src,
  MochiImage2Handle dst
);

// Shader Management
MochiShaderHandle mochi_rhi_shader_new(
  MochiDeviceHandle device,
  uint32_t stage,
  const uint8_t* code_ptr,
  size_t code_size,
  const char* entry
);
void              mochi_rhi_shader_dis(MochiShaderHandle shader);

// Pipeline Management
MochiPipelineMetaHandle mochi_rhi_pipeline_meta_new(uint32_t push_stage_flags, uint32_t push_size);
void                    mochi_rhi_pipeline_meta_dis(MochiPipelineMetaHandle meta);

MochiPipelineHandle     mochi_rhi_pipeline_graphics_new(
  MochiDeviceHandle device,
  MochiPipelineMetaHandle meta,
  const MochiShaderHandle* shaders,
  uint32_t shader_count,
  uint8_t polygon_mode,
  uint8_t primitive_topology,
  uint32_t color_format,
  uint32_t depth_format
);
void                    mochi_rhi_pipeline_dis(MochiPipelineHandle pipeline);



// Audio Device Management (Miniaudio)
typedef void (*MochiAudioCallback)(void* user_data, float* output_buffer, uint32_t frame_count);

MochiAudioDeviceHandle mochi_ahi_device_create(const char* app_name, uint16_t major, uint16_t minor, uint16_t patch, uint16_t build);
void                   mochi_ahi_device_destroy(MochiAudioDeviceHandle device);
MochiAudioDeviceHandle mochi_ahi_device_new(const char* app_name, uint16_t major, uint16_t minor, uint16_t patch, uint16_t build);
void                   mochi_ahi_device_dis(MochiAudioDeviceHandle device);
void                   mochi_ahi_device_set_callback(MochiAudioDeviceHandle device, MochiAudioCallback callback, void* user_data);
bool                   mochi_ahi_device_start(MochiAudioDeviceHandle device);
void                   mochi_ahi_device_stop(MochiAudioDeviceHandle device);
bool                   mochi_ahi_device_is_playing(MochiAudioDeviceHandle device);

// Audio Decoding
typedef struct MochiAudioData {
  uint32_t sample_rate;
  uint8_t channels;
  uint64_t frame_count;
  float* pcm_data;
} MochiAudioData;

bool mochi_ahi_device_decode_file(MochiAudioDeviceHandle device, const char* file_path, MochiAudioData* out_data);
bool mochi_ahi_device_decode_memory(MochiAudioDeviceHandle device, const uint8_t* memory, size_t size, MochiAudioData* out_data);
void mochi_ahi_free_audio_data(MochiAudioData* data);

#ifdef __cplusplus
}
#endif
