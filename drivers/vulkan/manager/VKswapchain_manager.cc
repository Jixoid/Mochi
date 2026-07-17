/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/

#include "drivers/vulkan/manager/VKswapchain_manager.hh"
#include "drivers/vulkan/VKdriver.hh"
#include "drivers/vulkan/VKrender_target.hh"
#include "drivers/vulkan/manager/VKdevice_manager.hh"
#include <format>
#include <GLFW/glfw3.h>
#include <vulkan/vulkan.hpp>
#include "mochi/debug/debug.hh"
#include "mochi/except.hh"
#include "mochi/rhi/render_target.hh"



namespace mochi::rhi::vulkan
{
  extern "C" fun MochiRHI_MakeSwapchainManager(rhi::DeviceManager &dmng) -> SwapchainManager* {
    return new VK_SwapchainManager(dmng);
  }


  
  VK_SwapchainManager::VK_SwapchainManager(rhi::DeviceManager &dmng)
    : rhi::SwapchainManager(dmng)
  {}

  VK_SwapchainManager::~VK_SwapchainManager() {
    destroy_resources();
  }

  void VK_SwapchainManager::destroy_resources() {
    auto& vk_dmng = static_cast<VK_DeviceManager&>(m_dmng);
    m_depth_view.clear();
    m_depth_image_handle.clear();
    m_depth_memory.clear();
  }

  void VK_SwapchainManager::create_resources() {
    auto& vk_dmng = static_cast<VK_DeviceManager&>(m_dmng);
    vk_dmng.get().waitIdle();
    
    destroy_resources();

    vk::SwapchainKHR old_swapchain = *m_swapchain;
    auto caps = vk_dmng.phys_dev().getSurfaceCapabilitiesKHR(m_surface);
    auto formats = vk_dmng.phys_dev().getSurfaceFormatsKHR(m_surface);

    vk::SurfaceFormatKHR selected_format = formats[0];
    for (const auto &f: formats) {
      if (f.format == vk::Format::eR8G8B8A8Unorm && f.colorSpace == vk::ColorSpaceKHR::eSrgbNonlinear) {
        selected_format = f; break;
      }
    }
    m_format = selected_format.format;

    if (caps.currentExtent.width != std::numeric_limits<u32>::max())
      m_extent = caps.currentExtent;
    else {
      m_extent.width = std::clamp(m_width, caps.minImageExtent.width, caps.maxImageExtent.width);
      m_extent.height = std::clamp(m_height, caps.minImageExtent.height, caps.maxImageExtent.height);
    }

    u32 m_image_count = std::max<u32>(caps.minImageCount, 3);
    if (caps.maxImageCount > 0 && m_image_count > caps.maxImageCount) {
      m_image_count = caps.maxImageCount;
    }

    vk::SwapchainCreateInfoKHR swp_info(
      {}, m_surface, m_image_count, m_format,
      selected_format.colorSpace, m_extent, 1,
      vk::ImageUsageFlagBits::eColorAttachment
    );
    
    auto present_modes = vk_dmng.phys_dev().getSurfacePresentModesKHR(m_surface);
    vk::PresentModeKHR present_mode = vk::PresentModeKHR::eFifo;
    for (const auto& mode : present_modes) {
      if (mode == vk::PresentModeKHR::eMailbox) { present_mode = mode; break; }
      else if (mode == vk::PresentModeKHR::eImmediate) { present_mode = mode; }
    }
    swp_info.setPresentMode(present_mode);
    swp_info.setPreTransform(caps.currentTransform);
    swp_info.setCompositeAlpha(vk::CompositeAlphaFlagBitsKHR::eOpaque);
    swp_info.setOldSwapchain(old_swapchain);

    m_swapchain = vk::raii::SwapchainKHR(vk_dmng.get(), swp_info);
    m_images = m_swapchain.getImages();
    m_image_views.clear();
    m_render_finished_sems.clear();

    for (const auto &img: m_images) {
      vk::ImageViewCreateInfo view_info(
        {}, img, vk::ImageViewType::e2D, m_format,
        {}, {vk::ImageAspectFlagBits::eColor, 0, 1, 0, 1}
      );
      m_image_views.push_back(vk::raii::ImageView(vk_dmng.get(), view_info));
      m_render_finished_sems.push_back(vk::raii::Semaphore(vk_dmng.get(), vk::SemaphoreCreateInfo()));
    }

    // Depth buffer allocation
    vk::ImageCreateInfo img_info(
      {}, vk::ImageType::e2D, m_depth_format,
      vk::Extent3D(m_extent.width, m_extent.height, 1),
      1, 1, vk::SampleCountFlagBits::e1, vk::ImageTiling::eOptimal,
      vk::ImageUsageFlagBits::eDepthStencilAttachment | vk::ImageUsageFlagBits::eSampled
    );
    m_depth_image_handle = vk::raii::Image(vk_dmng.get(), img_info);
    
    auto mem_req = m_depth_image_handle.getMemoryRequirements();
    auto mem_props = vk_dmng.phys_dev().getMemoryProperties();
    u32 mem_type_idx = 0;
    for (u32 i = 0; i < mem_props.memoryTypeCount; i++) {
      if ((mem_req.memoryTypeBits & (1 << i)) &&
          (mem_props.memoryTypes[i].propertyFlags & vk::MemoryPropertyFlagBits::eDeviceLocal) == vk::MemoryPropertyFlagBits::eDeviceLocal) {
        mem_type_idx = i;
        break;
      }
    }
    
    vk::MemoryAllocateInfo alloc_info(mem_req.size, mem_type_idx);
    m_depth_memory = vk::raii::DeviceMemory(vk_dmng.get(), alloc_info);
    m_depth_image_handle.bindMemory(*m_depth_memory, 0);

    vk::ImageViewCreateInfo view_info(
      {}, *m_depth_image_handle, vk::ImageViewType::e2D, m_depth_format,
      {}, {vk::ImageAspectFlagBits::eDepth, 0, 1, 0, 1}
    );
    m_depth_view = vk::raii::ImageView(vk_dmng.get(), view_info);

    m_targets.clear();
    for (size_t i = 0; i < m_images.size(); i++) {
      VK_RenderTarget rtg{
        m_images[i], *m_image_views[i], m_format,
        *m_depth_image_handle, *m_depth_view, m_depth_format,
        m_extent, vk::ImageLayout::ePresentSrcKHR
      };

      m_targets.push_back(std::move(rtg));
    }

    ME_LOG_VERB("swapchain (re)created {}x{}, {} images", m_extent.width, m_extent.height, m_images.size());
  }

  fun VK_SwapchainManager::init(void* windowHandle, u32 width, u32 height) -> void {
    auto& vk_dmng = static_cast<VK_DeviceManager&>(m_dmng);
    m_width = width;
    m_height = height;

    VkSurfaceKHR c_surface;
    if (glfwCreateWindowSurface(*vk_dmng.inst(), static_cast<GLFWwindow*>(windowHandle), nullptr, &c_surface) != VK_SUCCESS) {
      throw rhi_error("Failed to create window surface");
    }
    m_surface = vk::raii::SurfaceKHR(vk_dmng.inst(), c_surface);

    create_resources();
  }

  fun VK_SwapchainManager::resize(u32 width, u32 height) -> void {
    ME_LOG_VERB("swapchain resize {}x{}", width, height);
    m_width = width;
    m_height = height;
    create_resources();
  }

  fun VK_SwapchainManager::acquireNextImage(void* signalSemaphore) -> u32 {
    try {
      auto [result, img_idx] = m_swapchain.acquireNextImage(UINT64_MAX, static_cast<VkSemaphore>(signalSemaphore), nullptr);
      if (result == vk::Result::eErrorOutOfDateKHR || result == vk::Result::eSuboptimalKHR) {
        ME_LOG_WARN("swapchain out of date, recreating")
        create_resources();
        return acquireNextImage(signalSemaphore);
      }
      return img_idx;
    }
    catch (const vk::OutOfDateKHRError &) {
      ME_LOG_WARN("swapchain out of date, recreating")
      create_resources();
      return acquireNextImage(signalSemaphore);
    }
  }

  fun VK_SwapchainManager::present(u32 imageIndex, void* waitSemaphore) -> void {
    auto& vk_dmng = static_cast<VK_DeviceManager&>(m_dmng);
    
    vk::Semaphore sem = static_cast<VkSemaphore>(waitSemaphore);
    vk::SwapchainKHR swp = *m_swapchain;
    
    vk::PresentInfoKHR present_info;
    present_info.swapchainCount = 1;
    present_info.pSwapchains = &swp;
    present_info.pImageIndices = &imageIndex;
    if (waitSemaphore) {
      present_info.waitSemaphoreCount = 1;
      present_info.pWaitSemaphores = &sem;
    } else {
      present_info.waitSemaphoreCount = 0;
      present_info.pWaitSemaphores = nullptr;
    }

    try {
      auto family = vk_dmng.graphics_q().best().family();
      auto& queue = vk_dmng.active_queue(family);
      auto err = queue.presentKHR(present_info);
      if (err == vk::Result::eErrorOutOfDateKHR || err == vk::Result::eSuboptimalKHR) {
        create_resources();
      }
    } catch (const vk::OutOfDateKHRError &) {
      create_resources();
    }
  }

  fun VK_SwapchainManager::getRenderTarget(u32 index) -> RenderTarget& {
    return m_targets[index];
  }

  fun VK_SwapchainManager::getRenderTargetCount() const -> u32 {
    return m_targets.size();
  }

  fun VK_SwapchainManager::getRenderFinishedSemaphore(u32 image_index) -> void* {
    if (m_render_finished_sems.empty()) return nullptr;
    return (void*)(VkSemaphore)(*m_render_finished_sems[image_index]);
  }
}
