/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/

#include "drivers/vulkan/manager/VKswapchain_manager.hh"
#include "drivers/vulkan/manager/VKdevice_manager.hh"
#include <GLFW/glfw3.h>
#include <vulkan/vulkan.hpp>
#include "mochi/except.hh"



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
    // Depth image is not managed by RAII, we need VMA for it, but for VMA we need the Allocator.
    // However, AllocManager owns the allocator. We should really use AllocManager for depth buffer!
    // Since we need to destroy it, for now we will assume the caller handles or we manage it carefully.
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

    for (const auto &img: m_images) {
      vk::ImageViewCreateInfo view_info(
        {}, img, vk::ImageViewType::e2D, m_format,
        {}, {vk::ImageAspectFlagBits::eColor, 0, 1, 0, 1}
      );
      m_image_views.push_back(vk::raii::ImageView(vk_dmng.get(), view_info));
    }

    // Creating depth buffer should ideally use rhi::AllocManager but keeping simple mapping here
    // In strict RHI we'd ask AllocManager for a Depth Image!
    
    m_targets.clear();
    for (size_t i = 0; i < m_images.size(); i++) {
      m_targets.push_back(rhi::render_target{
        m_images[i], *m_image_views[i], m_format,
        m_depth_image, *m_depth_view, m_depth_format,
        m_extent, vk::ImageLayout::ePresentSrcKHR
      });
    }
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
    m_width = width;
    m_height = height;
    create_resources();
  }

  fun VK_SwapchainManager::acquireNextImage(void* signalSemaphore) -> u32 {
    try {
      auto [result, img_idx] = m_swapchain.acquireNextImage(UINT64_MAX, static_cast<VkSemaphore>(signalSemaphore), nullptr);
      if (result == vk::Result::eErrorOutOfDateKHR || result == vk::Result::eSuboptimalKHR) {
        create_resources();
        return acquireNextImage(signalSemaphore);
      }
      return img_idx;
    } catch (const vk::OutOfDateKHRError &) {
      create_resources();
      return acquireNextImage(signalSemaphore);
    }
  }

  fun VK_SwapchainManager::present(u32 imageIndex, void* waitSemaphore) -> void {
    auto& vk_dmng = static_cast<VK_DeviceManager&>(m_dmng);
    
    vk::Semaphore sem = static_cast<VkSemaphore>(waitSemaphore);
    vk::SwapchainKHR swp = *m_swapchain;
    
    vk::PresentInfoKHR present_info(1, &sem, 1, &swp, &imageIndex);

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

  fun VK_SwapchainManager::getRenderTarget(u32 index) -> render_target& {
    return m_targets[index];
  }

  fun VK_SwapchainManager::getRenderTargetCount() const -> u32 {
    return m_targets.size();
  }
}
