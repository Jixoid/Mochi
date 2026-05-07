/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#include "mochi/module/device.hh"
#include "mochi/module/window.hh"
#include "mochi/module/swapchain.hh"
#include <algorithm>
#include <vulkan/vulkan_raii.hpp>



namespace mochi::module
{

  swapchain::swapchain(device &device, window &window)
    : m_device(device)
    , m_window(window)
  {
    recreate();
  }



  fun swapchain::recreate() -> void
  {
    // Wait if window is minimized
    int width = 0, height = 0;
    glfwGetFramebufferSize(m_window.glfw(), &width, &height);
    while (width == 0 || height == 0) {
      glfwGetFramebufferSize(m_window.glfw(), &width, &height);
      glfwWaitEvents();
    }

    m_device.vdevice().waitIdle();


    m_image_views.clear();
    m_depth_view.clear();
    m_depth_image.clear();
    m_depth_memory.clear();


    // Smooth transition
    vk::SwapchainKHR old_swapchain = *m_swapchain;


    auto caps = m_device.phys_dev().getSurfaceCapabilitiesKHR(m_window.surface());
    auto formats = m_device.phys_dev().getSurfaceFormatsKHR(m_window.surface());


    vk::SurfaceFormatKHR selected_format = formats[0];
    for (const auto &f: formats)
      if (f.format == vk::Format::eB8G8R8A8Unorm && f.colorSpace == vk::ColorSpaceKHR::eSrgbNonlinear) {
        selected_format = f;
        break;
      }

    m_format = selected_format.format;



    if (caps.currentExtent.width != std::numeric_limits<u32>::max())
      m_extent = caps.currentExtent;
    else {
      m_extent.width = std::clamp(static_cast<u32>(width), caps.minImageExtent.width, caps.maxImageExtent.width);
      m_extent.height = std::clamp(static_cast<u32>(height), caps.minImageExtent.height, caps.maxImageExtent.height);
    }


    m_image_count = std::max<u32>(caps.minImageCount, 3);
    if (caps.maxImageCount > 0 && m_image_count > caps.maxImageCount) {
      m_image_count = caps.maxImageCount;
    }

    vk::SwapchainCreateInfoKHR swp_info(
      {}, m_window.surface(), m_image_count, m_format,
      selected_format.colorSpace, m_extent, 1,
      vk::ImageUsageFlagBits::eColorAttachment
    );
    
    swp_info.setPresentMode(vk::PresentModeKHR::eFifo);
    swp_info.setPreTransform(caps.currentTransform);
    swp_info.setCompositeAlpha(vk::CompositeAlphaFlagBitsKHR::eOpaque);
    swp_info.setOldSwapchain(old_swapchain);

    m_swapchain = vk::raii::SwapchainKHR(m_device.vdevice(), swp_info);


    m_images = m_swapchain.getImages();
    for (const auto &img: m_images) {
      vk::ImageViewCreateInfo view_info(
        {}, img, vk::ImageViewType::e2D, m_format,
        {}, {vk::ImageAspectFlagBits::eColor, 0, 1, 0, 1}
      );
      m_image_views.push_back(vk::raii::ImageView(m_device.vdevice(), view_info));
    }


    vk::ImageCreateInfo depth_info(
      {}, vk::ImageType::e2D, m_depth_format,
      vk::Extent3D(m_extent.width, m_extent.height, 1),
      1, 1, vk::SampleCountFlagBits::e1, vk::ImageTiling::eOptimal,
      vk::ImageUsageFlagBits::eDepthStencilAttachment
    );
    m_depth_image = vk::raii::Image(m_device.vdevice(), depth_info);


    vk::MemoryRequirements mem_reqs = m_depth_image.getMemoryRequirements();
    vk::PhysicalDeviceMemoryProperties mem_props = m_device.phys_dev().getMemoryProperties();
    u32 mem_type_idx = 0;
    
    for (u32 i = 0; i < mem_props.memoryTypeCount; i++) {
      if ((mem_reqs.memoryTypeBits & (1 << i)) &&
          (mem_props.memoryTypes[i].propertyFlags & vk::MemoryPropertyFlagBits::eDeviceLocal) == vk::MemoryPropertyFlagBits::eDeviceLocal) {
        mem_type_idx = i;
        break;
      }
    }

    vk::MemoryAllocateInfo alloc_info(mem_reqs.size, mem_type_idx);
    m_depth_memory = vk::raii::DeviceMemory(m_device.vdevice(), alloc_info);
    m_depth_image.bindMemory(*m_depth_memory, 0);


    vk::ImageViewCreateInfo depth_view_info(
      {}, *m_depth_image, vk::ImageViewType::e2D, m_depth_format,
      {}, {vk::ImageAspectFlagBits::eDepth, 0, 1, 0, 1}
    );
    m_depth_view = vk::raii::ImageView(m_device.vdevice(), depth_view_info);


    m_window.resized() = false;
  }

}
