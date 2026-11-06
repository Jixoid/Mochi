/*
  This file is part of QAOS / Mochi

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/

#include "mochi/driver_abi.h"
#include "drivers/vulkan/manager/VKdevice.hh"
#include "drivers/vulkan/manager/VKallocator.hh"
#include "drivers/vulkan/VKbuffer.hh"
#include "drivers/vulkan/VKimage.hh"
#include "drivers/vulkan/VKsampler.hh"
#include "drivers/vulkan/manager/VKswapchain_manager.hh"
#include "drivers/vulkan/manager/VKsynchronizer.hh"
#include "drivers/vulkan/manager/VKcommand_manager.hh"
#include "drivers/vulkan/VKcommand.hh"
#include "drivers/vulkan/VKrender_target.hh"
#include "drivers/vulkan/manager/VKuploader.hh"
#include "drivers/vulkan/VKshader.hh"
#include "drivers/vulkan/VKpipeline.hh"
#include "drivers/vulkan/utility/VKpipeline_cache_utility.hh"

#if defined(__linux__)
#include <wayland-client.h>
#include <vulkan/vulkan_wayland.h>
#include <X11/Xlib.h>
#include <vulkan/vulkan_xlib.h>
#include <xcb/xcb.h>
#include <vulkan/vulkan_xcb.h>
#elif defined(_WIN32)
#include <windows.h>
#include <vulkan/vulkan_win32.h>
#endif

using namespace mochi;

extern "C" {

const char* mochi_rhi_get_driver_name(void) {
  return "Vulkan 1.4";
}

void mochi_rhi_get_driver_version(uint32_t out_version[4]) {
  if (out_version) {
    out_version[0] = 1;
    out_version[1] = 4;
    out_version[2] = 0;
    out_version[3] = 0;
  }
}

MochiDeviceHandle mochi_rhi_device_create(const char* app_name, uint16_t major, uint16_t minor, uint16_t patch, uint16_t build) {
  auto dev = new rhi::vulkan::VK_Device(app_name ? app_name : "MochiApp", {major, minor, patch, build});
  return reinterpret_cast<MochiDeviceHandle>(dev);
}

void mochi_rhi_device_destroy(MochiDeviceHandle device) {
  if (device) {
    delete reinterpret_cast<rhi::vulkan::VK_Device*>(device);
  }
}

MochiDeviceHandle mochi_rhi_device_new(const char* app_name, uint16_t major, uint16_t minor, uint16_t patch, uint16_t build) {
  return mochi_rhi_device_create(app_name, major, minor, patch, build);
}

void mochi_rhi_device_dis(MochiDeviceHandle device) {
  mochi_rhi_device_destroy(device);
}

void mochi_rhi_device_wait_idle(MochiDeviceHandle device) {
  if (device) {
    reinterpret_cast<rhi::vulkan::VK_Device*>(device)->waitIdle();
  }
}

uint32_t mochi_rhi_device_get_main_queue_family(MochiDeviceHandle device) {
  if (!device) return 0;
  return reinterpret_cast<rhi::vulkan::VK_Device*>(device)->main_q().family();
}

void mochi_rhi_device_init_descriptor_heap(MochiDeviceHandle device, MochiAllocatorHandle allocator) {
  if (device && allocator) {
    auto dev = reinterpret_cast<rhi::vulkan::VK_Device*>(device);
    auto alloc = reinterpret_cast<rhi::vulkan::VK_Allocator*>(allocator);
    dev->initDescriptorHeap(*alloc);
  }
}

void mochi_rhi_device_fini_descriptor_heap(MochiDeviceHandle device) {
  if (device) {
    reinterpret_cast<rhi::vulkan::VK_Device*>(device)->finiDescriptorHeap();
  }
}

uint32_t mochi_rhi_device_write_texture_descriptor(MochiDeviceHandle device, MochiImage2Handle view, MochiSampler2Handle sampler) {
  if (!device || !view || !sampler) return 0;
  auto dev = reinterpret_cast<rhi::vulkan::VK_Device*>(device);
  auto img = *reinterpret_cast<sptr<rhi::Image2>*>(view);
  auto samp = *reinterpret_cast<sptr<rhi::Sampler2>*>(sampler);
  return dev->writeTextureDescriptor(img->makeView(), samp);
}

MochiBufferHandle mochi_rhi_device_get_descriptor_heap(MochiDeviceHandle device) {
  if (!device) return nullptr;
  auto dev = reinterpret_cast<rhi::vulkan::VK_Device*>(device);
  auto heap = dev->descriptor_heap();
  return heap ? reinterpret_cast<MochiBufferHandle>(new sptr<rhi::Buffer>(heap)) : nullptr;
}

MochiBufferHandle mochi_rhi_device_get_sampler_heap(MochiDeviceHandle device) {
  if (!device) return nullptr;
  auto dev = reinterpret_cast<rhi::vulkan::VK_Device*>(device);
  auto heap = dev->sampler_heap();
  return heap ? reinterpret_cast<MochiBufferHandle>(new sptr<rhi::Buffer>(heap)) : nullptr;
}

MochiAllocatorHandle mochi_rhi_allocator_create(MochiDeviceHandle device) {
  if (!device) return nullptr;
  auto dev = reinterpret_cast<rhi::vulkan::VK_Device*>(device);
  auto alloc = new rhi::vulkan::VK_Allocator(*dev);
  return reinterpret_cast<MochiAllocatorHandle>(alloc);
}

void mochi_rhi_allocator_destroy(MochiAllocatorHandle allocator) {
  if (allocator) {
    delete reinterpret_cast<rhi::vulkan::VK_Allocator*>(allocator);
  }
}

MochiAllocatorHandle mochi_rhi_allocator_new(MochiDeviceHandle device) {
  return mochi_rhi_allocator_create(device);
}

void mochi_rhi_allocator_dis(MochiAllocatorHandle allocator) {
  mochi_rhi_allocator_destroy(allocator);
}

MochiBufferHandle mochi_rhi_allocator_alloc_buffer(
    MochiAllocatorHandle allocator,
    uint64_t size,
    uint32_t usage_flags,
    uint32_t create_flags,
    uint32_t location
) {
  if (!allocator) return nullptr;
  auto alloc = reinterpret_cast<rhi::vulkan::VK_Allocator*>(allocator);
  auto buf = alloc->allocBuffer(
    size,
    static_cast<rhi::BufferUsageFlags>(usage_flags),
    static_cast<rhi::AllocationCreateFlags>(create_flags),
    static_cast<rhi::AllocationLocation>(location)
  );
  return buf ? reinterpret_cast<MochiBufferHandle>(new sptr<rhi::Buffer>(buf)) : nullptr;
}

MochiImage2Handle mochi_rhi_allocator_alloc_image2(
    MochiAllocatorHandle allocator,
    uint32_t width,
    uint32_t height,
    uint32_t format,
    uint32_t usage_flags,
    uint32_t tiling,
    uint32_t create_flags,
    uint32_t location
) {
  if (!allocator) return nullptr;
  auto alloc = reinterpret_cast<rhi::vulkan::VK_Allocator*>(allocator);
  auto img = alloc->allocImage2(
    extent<2, u32>{width, height},
    static_cast<rhi::Format>(format),
    static_cast<rhi::ImageUsageFlags>(usage_flags),
    static_cast<rhi::ImageTiling>(tiling),
    static_cast<rhi::AllocationCreateFlags>(create_flags),
    static_cast<rhi::AllocationLocation>(location)
  );
  return img ? reinterpret_cast<MochiImage2Handle>(new sptr<rhi::Image2>(img)) : nullptr;
}

MochiSampler2Handle mochi_rhi_allocator_alloc_sampler2(
    MochiAllocatorHandle allocator,
    uint32_t address_mode_u,
    uint32_t address_mode_v,
    uint32_t address_mode_w,
    uint32_t mag_filter,
    uint32_t min_filter
) {
  if (!allocator) return nullptr;
  auto alloc = reinterpret_cast<rhi::vulkan::VK_Allocator*>(allocator);
  auto samp = alloc->allocSampler2(
    extent<3, rhi::SamplerAddressMode>{
      static_cast<rhi::SamplerAddressMode>(address_mode_u),
      static_cast<rhi::SamplerAddressMode>(address_mode_v),
      static_cast<rhi::SamplerAddressMode>(address_mode_w)
    },
    static_cast<rhi::SamplerFilter>(mag_filter),
    static_cast<rhi::SamplerFilter>(min_filter)
  );
  return samp ? reinterpret_cast<MochiSampler2Handle>(new sptr<rhi::Sampler2>(samp)) : nullptr;
}

bool mochi_rhi_allocator_support_statistics(MochiAllocatorHandle allocator) {
  if (!allocator) return false;
  return reinterpret_cast<rhi::vulkan::VK_Allocator*>(allocator)->support_statistics();
}

uint64_t mochi_rhi_allocator_stat_total_vram(MochiAllocatorHandle allocator) {
  if (!allocator) return 0;
  return reinterpret_cast<rhi::vulkan::VK_Allocator*>(allocator)->stat_total_vram();
}

uint64_t mochi_rhi_allocator_stat_cache_vram(MochiAllocatorHandle allocator) {
  if (!allocator) return 0;
  return reinterpret_cast<rhi::vulkan::VK_Allocator*>(allocator)->stat_cache_vram();
}

uint64_t mochi_rhi_allocator_stat_used_vram(MochiAllocatorHandle allocator) {
  if (!allocator) return 0;
  return reinterpret_cast<rhi::vulkan::VK_Allocator*>(allocator)->stat_used_vram();
}

uint64_t mochi_rhi_allocator_stat_live_buffer(MochiAllocatorHandle allocator) {
  if (!allocator) return 0;
  return reinterpret_cast<rhi::vulkan::VK_Allocator*>(allocator)->stat_live_buffer();
}

uint64_t mochi_rhi_allocator_stat_live_image2(MochiAllocatorHandle allocator) {
  if (!allocator) return 0;
  return reinterpret_cast<rhi::vulkan::VK_Allocator*>(allocator)->stat_live_image2();
}

void mochi_rhi_buffer_destroy(MochiBufferHandle buffer) {
  if (buffer) {
    delete reinterpret_cast<sptr<rhi::Buffer>*>(buffer);
  }
}

void mochi_rhi_buffer_dis(MochiBufferHandle buffer) {
  mochi_rhi_buffer_destroy(buffer);
}

uint64_t mochi_rhi_buffer_get_size(MochiBufferHandle buffer) {
  if (!buffer) return 0;
  return (*reinterpret_cast<sptr<rhi::Buffer>*>(buffer))->size();
}

void mochi_rhi_buffer_map(MochiBufferHandle buffer) {
  if (buffer) {
    (*reinterpret_cast<sptr<rhi::Buffer>*>(buffer))->map();
  }
}

void mochi_rhi_buffer_unmap(MochiBufferHandle buffer) {
  if (buffer) {
    (*reinterpret_cast<sptr<rhi::Buffer>*>(buffer))->unmap();
  }
}

void mochi_rhi_buffer_flush(MochiBufferHandle buffer, uint64_t offset, uint64_t size) {
  if (buffer) {
    (*reinterpret_cast<sptr<rhi::Buffer>*>(buffer))->flush(offs(offset, size));
  }
}

void* mochi_rhi_buffer_get_mapped_ptr(MochiBufferHandle buffer) {
  if (!buffer) return nullptr;
  return (*reinterpret_cast<sptr<rhi::Buffer>*>(buffer))->mapped();
}

uint64_t mochi_rhi_buffer_get_device_address(MochiBufferHandle buffer) {
  if (!buffer) return 0;
  return (*reinterpret_cast<sptr<rhi::Buffer>*>(buffer))->address();
}

void mochi_rhi_image2_destroy(MochiImage2Handle image) {
  if (image) {
    delete reinterpret_cast<sptr<rhi::Image2>*>(image);
  }
}

void mochi_rhi_image2_dis(MochiImage2Handle image) {
  mochi_rhi_image2_destroy(image);
}

void mochi_rhi_sampler2_destroy(MochiSampler2Handle sampler) {
  if (sampler) {
    delete reinterpret_cast<sptr<rhi::Sampler2>*>(sampler);
  }
}

void mochi_rhi_sampler2_dis(MochiSampler2Handle sampler) {
  mochi_rhi_sampler2_destroy(sampler);
}

uint64_t mochi_rhi_create_surface_wayland(MochiDeviceHandle device, void* display, void* surface) {
#if defined(__linux__)
  if (!device || !display || !surface) return 0;
  auto dev = reinterpret_cast<rhi::vulkan::VK_Device*>(device);

  VkWaylandSurfaceCreateInfoKHR createInfo{};
  createInfo.sType = VK_STRUCTURE_TYPE_WAYLAND_SURFACE_CREATE_INFO_KHR;
  createInfo.display = static_cast<struct wl_display*>(display);
  createInfo.surface = static_cast<struct wl_surface*>(surface);

  VkSurfaceKHR c_surface = VK_NULL_HANDLE;
  auto func = reinterpret_cast<PFN_vkCreateWaylandSurfaceKHR>(
    dev->inst().getDispatcher()->vkGetInstanceProcAddr(*dev->inst(), "vkCreateWaylandSurfaceKHR")
  );
  if (!func || func(*dev->inst(), &createInfo, nullptr, &c_surface) != VK_SUCCESS) {
    return 0;
  }
  return reinterpret_cast<uint64_t>(c_surface);
#else
  (void)device; (void)display; (void)surface;
  return 0;
#endif
}

uint64_t mochi_rhi_create_surface_xlib(MochiDeviceHandle device, void* display, unsigned long window) {
#if defined(__linux__)
  if (!device || !display) return 0;
  auto dev = reinterpret_cast<rhi::vulkan::VK_Device*>(device);

  VkXlibSurfaceCreateInfoKHR createInfo{};
  createInfo.sType = VK_STRUCTURE_TYPE_XLIB_SURFACE_CREATE_INFO_KHR;
  createInfo.dpy = static_cast<Display*>(display);
  createInfo.window = window;

  VkSurfaceKHR c_surface = VK_NULL_HANDLE;
  auto func = reinterpret_cast<PFN_vkCreateXlibSurfaceKHR>(
    dev->inst().getDispatcher()->vkGetInstanceProcAddr(*dev->inst(), "vkCreateXlibSurfaceKHR")
  );
  if (!func || func(*dev->inst(), &createInfo, nullptr, &c_surface) != VK_SUCCESS) {
    return 0;
  }
  return reinterpret_cast<uint64_t>(c_surface);
#else
  (void)device; (void)display; (void)window;
  return 0;
#endif
}

uint64_t mochi_rhi_create_surface_xcb(MochiDeviceHandle device, void* connection, uint32_t window) {
#if defined(__linux__)
  if (!device || !connection) return 0;
  auto dev = reinterpret_cast<rhi::vulkan::VK_Device*>(device);

  VkXcbSurfaceCreateInfoKHR createInfo{};
  createInfo.sType = VK_STRUCTURE_TYPE_XCB_SURFACE_CREATE_INFO_KHR;
  createInfo.connection = static_cast<xcb_connection_t*>(connection);
  createInfo.window = window;

  VkSurfaceKHR c_surface = VK_NULL_HANDLE;
  auto func = reinterpret_cast<PFN_vkCreateXcbSurfaceKHR>(
    dev->inst().getDispatcher()->vkGetInstanceProcAddr(*dev->inst(), "vkCreateXcbSurfaceKHR")
  );
  if (!func || func(*dev->inst(), &createInfo, nullptr, &c_surface) != VK_SUCCESS) {
    return 0;
  }
  return reinterpret_cast<uint64_t>(c_surface);
#else
  (void)device; (void)connection; (void)window;
  return 0;
#endif
}

uint64_t mochi_rhi_create_surface_win32(MochiDeviceHandle device, void* hinstance, void* hwnd) {
#if defined(_WIN32)
  if (!device || !hwnd) return 0;
  auto dev = reinterpret_cast<rhi::vulkan::VK_Device*>(device);

  VkWin32SurfaceCreateInfoKHR createInfo{};
  createInfo.sType = VK_STRUCTURE_TYPE_WIN32_SURFACE_CREATE_INFO_KHR;
  createInfo.hinstance = static_cast<HINSTANCE>(hinstance);
  createInfo.hwnd = static_cast<HWND>(hwnd);

  VkSurfaceKHR c_surface = VK_NULL_HANDLE;
  auto func = reinterpret_cast<PFN_vkCreateWin32SurfaceKHR>(
    dev->inst().getDispatcher()->vkGetInstanceProcAddr(*dev->inst(), "vkCreateWin32SurfaceKHR")
  );
  if (!func || func(*dev->inst(), &createInfo, nullptr, &c_surface) != VK_SUCCESS) {
    return 0;
  }
  return reinterpret_cast<uint64_t>(c_surface);
#else
  (void)device; (void)hinstance; (void)hwnd;
  return 0;
#endif
}

void mochi_rhi_destroy_surface(MochiDeviceHandle device, uint64_t surface) {
  if (device && surface) {
    auto dev = reinterpret_cast<rhi::vulkan::VK_Device*>(device);
    auto s = reinterpret_cast<VkSurfaceKHR>(surface);
    auto func = reinterpret_cast<PFN_vkDestroySurfaceKHR>(
      dev->inst().getDispatcher()->vkGetInstanceProcAddr(*dev->inst(), "vkDestroySurfaceKHR")
    );
    if (func) {
      func(*dev->inst(), s, nullptr);
    }
  }
}

MochiSwapchainHandle mochi_rhi_swapchain_create(MochiDeviceHandle device, uint64_t surface, uint32_t width, uint32_t height) {
  if (!device || !surface) return nullptr;
  auto dev = reinterpret_cast<rhi::vulkan::VK_Device*>(device);
  auto smng = new rhi::vulkan::VK_SwapchainManager(*dev);
  smng->initWithSurface(reinterpret_cast<VkSurfaceKHR>(surface), width, height);
  return reinterpret_cast<MochiSwapchainHandle>(smng);
}

void mochi_rhi_swapchain_destroy(MochiSwapchainHandle swapchain) {
  if (swapchain) {
    delete reinterpret_cast<rhi::vulkan::VK_SwapchainManager*>(swapchain);
  }
}

MochiSwapchainHandle mochi_rhi_swapchain_new(MochiDeviceHandle device, uint64_t surface, uint32_t width, uint32_t height) {
  return mochi_rhi_swapchain_create(device, surface, width, height);
}

void mochi_rhi_swapchain_dis(MochiSwapchainHandle swapchain) {
  mochi_rhi_swapchain_destroy(swapchain);
}

void mochi_rhi_swapchain_resize(MochiSwapchainHandle swapchain, uint32_t width, uint32_t height) {
  if (swapchain) {
    reinterpret_cast<rhi::vulkan::VK_SwapchainManager*>(swapchain)->resize(width, height);
  }
}

uint32_t mochi_rhi_swapchain_acquire_next_image(MochiSwapchainHandle swapchain, void* signal_semaphore) {
  if (!swapchain) return 0;
  return reinterpret_cast<rhi::vulkan::VK_SwapchainManager*>(swapchain)->acquireNextImage(signal_semaphore);
}

void mochi_rhi_swapchain_present(MochiSwapchainHandle swapchain, uint32_t image_index, void* wait_semaphore) {
  if (swapchain) {
    reinterpret_cast<rhi::vulkan::VK_SwapchainManager*>(swapchain)->present(image_index, wait_semaphore);
  }
}

uint32_t mochi_rhi_swapchain_get_image_count(MochiSwapchainHandle swapchain) {
  if (!swapchain) return 0;
  return reinterpret_cast<rhi::vulkan::VK_SwapchainManager*>(swapchain)->getRenderTargetCount();
}

// ============================================================================
// Synchronizer Management
// ============================================================================
MochiSynchronizerHandle mochi_rhi_synchronizer_create(MochiDeviceHandle device, uint32_t max_frames_in_flight) {
  if (!device) return nullptr;
  auto dev = reinterpret_cast<rhi::vulkan::VK_Device*>(device);
  auto sync = new rhi::vulkan::VK_Synchronizer(*dev, max_frames_in_flight);
  return reinterpret_cast<MochiSynchronizerHandle>(sync);
}

void mochi_rhi_synchronizer_destroy(MochiSynchronizerHandle synchronizer) {
  if (synchronizer) {
    delete reinterpret_cast<rhi::vulkan::VK_Synchronizer*>(synchronizer);
  }
}

MochiSynchronizerHandle mochi_rhi_synchronizer_new(MochiDeviceHandle device, uint32_t max_frames_in_flight) {
  return mochi_rhi_synchronizer_create(device, max_frames_in_flight);
}

void mochi_rhi_synchronizer_dis(MochiSynchronizerHandle synchronizer) {
  mochi_rhi_synchronizer_destroy(synchronizer);
}

void mochi_rhi_synchronizer_begin_frame(MochiSynchronizerHandle synchronizer) {
  if (synchronizer) {
    reinterpret_cast<rhi::vulkan::VK_Synchronizer*>(synchronizer)->beginFrame();
  }
}

void mochi_rhi_synchronizer_end_frame(MochiSynchronizerHandle synchronizer) {
  if (synchronizer) {
    reinterpret_cast<rhi::vulkan::VK_Synchronizer*>(synchronizer)->endFrame();
  }
}

uint32_t mochi_rhi_synchronizer_current_frame(MochiSynchronizerHandle synchronizer) {
  if (!synchronizer) return 0;
  return reinterpret_cast<rhi::vulkan::VK_Synchronizer*>(synchronizer)->currentFrameIndex();
}

void* mochi_rhi_synchronizer_get_image_available_semaphore(MochiSynchronizerHandle synchronizer) {
  if (!synchronizer) return nullptr;
  return reinterpret_cast<rhi::vulkan::VK_Synchronizer*>(synchronizer)->activeImageAvailableSemaphore();
}

void* mochi_rhi_synchronizer_get_in_flight_fence(MochiSynchronizerHandle synchronizer) {
  if (!synchronizer) return nullptr;
  return reinterpret_cast<rhi::vulkan::VK_Synchronizer*>(synchronizer)->activeInFlightFence();
}

// ============================================================================
// RenderTarget & Swapchain RenderTarget
// ============================================================================
MochiRenderTargetHandle mochi_rhi_swapchain_get_render_target(MochiSwapchainHandle swapchain, uint32_t index) {
  if (!swapchain) return nullptr;
  auto smng = reinterpret_cast<rhi::vulkan::VK_SwapchainManager*>(swapchain);
  if (index >= smng->getRenderTargetCount()) return nullptr;
  return reinterpret_cast<MochiRenderTargetHandle>(&smng->getRenderTarget(index));
}

void* mochi_rhi_swapchain_get_render_finished_semaphore(MochiSwapchainHandle swapchain, uint32_t index) {
  if (!swapchain) return nullptr;
  auto smng = reinterpret_cast<rhi::vulkan::VK_SwapchainManager*>(swapchain);
  return smng->getRenderFinishedSemaphore(index);
}

uint32_t mochi_rhi_render_target_get_color_format(MochiRenderTargetHandle target) {
  if (!target) return 0;
  return static_cast<uint32_t>(reinterpret_cast<rhi::RenderTarget*>(target)->color_format());
}

uint32_t mochi_rhi_render_target_get_depth_format(MochiRenderTargetHandle target) {
  if (!target) return 0;
  return static_cast<uint32_t>(reinterpret_cast<rhi::RenderTarget*>(target)->depth_format());
}

// ============================================================================
// Command Manager & Command
// ============================================================================
MochiCommandManagerHandle mochi_rhi_command_manager_create(MochiDeviceHandle device) {
  if (!device) return nullptr;
  auto dev = reinterpret_cast<rhi::vulkan::VK_Device*>(device);
  auto cmng = new rhi::vulkan::VK_CommandManager(*dev);
  return reinterpret_cast<MochiCommandManagerHandle>(cmng);
}

void mochi_rhi_command_manager_destroy(MochiCommandManagerHandle manager) {
  if (manager) {
    delete reinterpret_cast<rhi::vulkan::VK_CommandManager*>(manager);
  }
}

MochiCommandManagerHandle mochi_rhi_command_manager_new(MochiDeviceHandle device) {
  return mochi_rhi_command_manager_create(device);
}

void mochi_rhi_command_manager_dis(MochiCommandManagerHandle manager) {
  mochi_rhi_command_manager_destroy(manager);
}

MochiCommandHandle mochi_rhi_command_manager_allocate_graphics(MochiCommandManagerHandle manager) {
  if (!manager) return nullptr;
  auto cmng = reinterpret_cast<rhi::vulkan::VK_CommandManager*>(manager);
  auto cmd = cmng->allocateGraphicsCommand();
  return cmd ? reinterpret_cast<MochiCommandHandle>(new sptr<rhi::Command>(cmd)) : nullptr;
}

void mochi_rhi_command_manager_submit(MochiCommandManagerHandle manager, MochiCommandHandle cmd) {
  if (manager && cmd) {
    auto cmng = reinterpret_cast<rhi::vulkan::VK_CommandManager*>(manager);
    auto c = *reinterpret_cast<sptr<rhi::Command>*>(cmd);
    cmng->submit(c.get());
  }
}

void mochi_rhi_command_manager_submit_with_sync(
    MochiCommandManagerHandle manager,
    MochiCommandHandle cmd,
    void* wait_semaphore,
    void* signal_semaphore,
    void* fence
) {
  if (manager && cmd) {
    auto cmng = reinterpret_cast<rhi::vulkan::VK_CommandManager*>(manager);
    auto c = *reinterpret_cast<sptr<rhi::Command>*>(cmd);
    cmng->submitWithSync(c.get(), wait_semaphore, signal_semaphore, fence);
  }
}

void mochi_rhi_command_destroy(MochiCommandHandle cmd) {
  if (cmd) {
    delete reinterpret_cast<sptr<rhi::Command>*>(cmd);
  }
}

void mochi_rhi_command_dis(MochiCommandHandle cmd) {
  mochi_rhi_command_destroy(cmd);
}

void mochi_rhi_command_begin(MochiCommandHandle cmd) {
  if (cmd) {
    (*reinterpret_cast<sptr<rhi::Command>*>(cmd))->begin();
  }
}

void mochi_rhi_command_end(MochiCommandHandle cmd) {
  if (cmd) {
    (*reinterpret_cast<sptr<rhi::Command>*>(cmd))->end();
  }
}

void mochi_rhi_command_begin_rendering(
    MochiCommandHandle cmd,
    MochiRenderTargetHandle target,
    float clear_r, float clear_g, float clear_b, float clear_a
) {
  if (cmd && target) {
    auto c = *reinterpret_cast<sptr<rhi::Command>*>(cmd);
    auto t = reinterpret_cast<rhi::RenderTarget*>(target);
    c->beginRendering(*t, {clear_r, clear_g, clear_b, clear_a});
  }
}

void mochi_rhi_command_end_rendering(MochiCommandHandle cmd, MochiRenderTargetHandle target) {
  if (cmd && target) {
    auto c = *reinterpret_cast<sptr<rhi::Command>*>(cmd);
    auto t = reinterpret_cast<rhi::RenderTarget*>(target);
    c->endRendering(*t);
  }
}

void mochi_rhi_command_set_viewport(
    MochiCommandHandle cmd,
    float x, float y, float width, float height, float min_depth, float max_depth
) {
  if (cmd) {
    auto c = *reinterpret_cast<sptr<rhi::Command>*>(cmd);
    rhi::Viewport vp(x, x + width, y, y + height, min_depth, max_depth);
    c->setViewport(0, {vp});
  }
}

void mochi_rhi_command_set_scissor(
    MochiCommandHandle cmd,
    int32_t x, int32_t y, uint32_t width, uint32_t height
) {
  if (cmd) {
    auto c = *reinterpret_cast<sptr<rhi::Command>*>(cmd);
    rhi::Rect2D sc(x, y, width, height);
    c->setScissor(0, {sc});
  }
}

void mochi_rhi_command_bind_pipeline(MochiCommandHandle cmd, MochiPipelineHandle pipeline) {
  if (cmd && pipeline) {
    auto c = *reinterpret_cast<sptr<rhi::Command>*>(cmd);
    auto p = reinterpret_cast<rhi::Pipeline*>(pipeline);
    c->bindPipeline(p);
  }
}

void mochi_rhi_command_bind_descriptor_heap(
    MochiCommandHandle cmd,
    MochiBufferHandle resource_heap,
    MochiBufferHandle sampler_heap,
    uint64_t resource_size,
    uint64_t sampler_size
) {
  if (cmd) {
    auto c = *reinterpret_cast<sptr<rhi::Command>*>(cmd);
    sptr<rhi::Buffer> res_sptr;
    if (resource_heap) {
      res_sptr = *reinterpret_cast<sptr<rhi::Buffer>*>(resource_heap);
    }
    sptr<rhi::Buffer> samp_sptr;
    if (sampler_heap) {
      samp_sptr = *reinterpret_cast<sptr<rhi::Buffer>*>(sampler_heap);
    }
    c->bindDescriptorHeap(res_sptr, samp_sptr, resource_size, sampler_size);
  }
}

void mochi_rhi_command_push_constant(
    MochiCommandHandle cmd,
    uint32_t stage_flags,
    uint32_t offset,
    uint32_t size,
    const void* data_ptr
) {
  if (cmd && data_ptr && size > 0) {
    auto c = *reinterpret_cast<sptr<rhi::Command>*>(cmd);
    c->pushConstant(
      static_cast<rhi::ShaderStageFlags>(stage_flags),
      offset,
      ::data(const_cast<void*>(data_ptr), size)
    );
  }
}

void mochi_rhi_command_draw(
    MochiCommandHandle cmd,
    uint32_t vertex_count,
    uint32_t instance_count,
    uint32_t first_vertex,
    uint32_t first_instance
) {
  if (cmd) {
    auto c = *reinterpret_cast<sptr<rhi::Command>*>(cmd);
    c->draw(::offs(first_vertex, vertex_count), ::offs(first_instance, instance_count));
  }
}

// ============================================================================
// Uploader
// ============================================================================
MochiUploaderHandle mochi_rhi_uploader_new(MochiDeviceHandle device) {
  if (!device) return nullptr;
  auto dev = reinterpret_cast<rhi::vulkan::VK_Device*>(device);
  auto uploader = new rhi::vulkan::VK_Uploader(*dev);
  return reinterpret_cast<MochiUploaderHandle>(uploader);
}

void mochi_rhi_uploader_dis(MochiUploaderHandle uploader) {
  if (uploader) {
    delete reinterpret_cast<rhi::vulkan::VK_Uploader*>(uploader);
  }
}

void mochi_rhi_uploader_copy_memory_to_buffer(
    MochiUploaderHandle uploader,
    uint32_t transfer_time,
    const void* src,
    MochiBufferHandle dst
) {
  if (uploader && src && dst) {
    auto up = reinterpret_cast<rhi::vulkan::VK_Uploader*>(uploader);
    auto buf = (*reinterpret_cast<sptr<rhi::Buffer>*>(dst)).get();
    up->copyMemoryToBuffer(static_cast<rhi::TransferTime>(transfer_time), const_cast<void*>(src), buf);
  }
}

void mochi_rhi_uploader_copy_memory_to_image(
    MochiUploaderHandle uploader,
    uint32_t transfer_time,
    const void* src,
    MochiImage2Handle dst
) {
  if (uploader && src && dst) {
    auto up = reinterpret_cast<rhi::vulkan::VK_Uploader*>(uploader);
    auto img = (*reinterpret_cast<sptr<rhi::Image2>*>(dst)).get();
    up->copyMemoryToImage(static_cast<rhi::TransferTime>(transfer_time), const_cast<void*>(src), img);
  }
}

// ============================================================================
// Shader
// ============================================================================
MochiShaderHandle mochi_rhi_shader_new(
    MochiDeviceHandle device,
    uint32_t stage,
    const uint8_t* code_ptr,
    size_t code_size,
    const char* entry
) {
  if (!device || !code_ptr || code_size == 0) return nullptr;
  auto dev = reinterpret_cast<rhi::vulkan::VK_Device*>(device);
  std::span<u8> span(const_cast<u8*>(code_ptr), code_size);
  auto sh = new rhi::vulkan::VK_Shader(
    *dev,
    static_cast<rhi::ShaderStage>(stage),
    span,
    entry ? entry : "main"
  );
  return reinterpret_cast<MochiShaderHandle>(sh);
}

void mochi_rhi_shader_dis(MochiShaderHandle shader) {
  if (shader) {
    delete reinterpret_cast<rhi::vulkan::VK_Shader*>(shader);
  }
}

// ============================================================================
// Pipeline
// ============================================================================
MochiPipelineMetaHandle mochi_rhi_pipeline_meta_new(uint32_t push_stage_flags, uint32_t push_size) {
  auto meta = new rhi::vulkan::VK_PipelineMeta(
    vk::ShaderStageFlags(push_stage_flags),
    push_size
  );
  return reinterpret_cast<MochiPipelineMetaHandle>(meta);
}

void mochi_rhi_pipeline_meta_dis(MochiPipelineMetaHandle meta) {
  if (meta) {
    delete reinterpret_cast<rhi::vulkan::VK_PipelineMeta*>(meta);
  }
}

MochiPipelineHandle mochi_rhi_pipeline_graphics_new(
    MochiDeviceHandle device,
    MochiPipelineMetaHandle meta,
    const MochiShaderHandle* shaders,
    uint32_t shader_count,
    uint8_t polygon_mode,
    uint8_t primitive_topology,
    uint32_t color_format,
    uint32_t depth_format
) {
  if (!device || !meta || !shaders || shader_count == 0) return nullptr;
  auto dev = reinterpret_cast<rhi::vulkan::VK_Device*>(device);
  auto pmeta = reinterpret_cast<rhi::vulkan::VK_PipelineMeta*>(meta);

  std::vector<sptr<rhi::Shader>> shader_ptrs;
  shader_ptrs.reserve(shader_count);
  for (uint32_t i = 0; i < shader_count; i++) {
    auto sh = reinterpret_cast<rhi::Shader*>(shaders[i]);
    shader_ptrs.emplace_back(sh, [](rhi::Shader*){});
  }

  rhi::vulkan::VK_PipelineCacheUtility cache_util(*dev);
  auto pipe = new rhi::vulkan::VK_Pipeline(
    *dev,
    cache_util,
    0, // sign 0 = no persistent cache file
    pmeta,
    std::move(shader_ptrs),
    static_cast<rhi::PolygonMode>(polygon_mode),
    static_cast<rhi::PrimitiveTopology>(primitive_topology),
    static_cast<rhi::Format>(color_format),
    static_cast<rhi::Format>(depth_format)
  );

  return reinterpret_cast<MochiPipelineHandle>(pipe);
}

void mochi_rhi_pipeline_dis(MochiPipelineHandle pipeline) {
  if (pipeline) {
    delete reinterpret_cast<rhi::vulkan::VK_Pipeline*>(pipeline);
  }
}

}

