/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#include "qvk/module/device.hh"
#include "qvk/module/window.hh"
#include "qvk/module/swapchain.hh"
#include <algorithm>
#include <vulkan/vulkan_raii.hpp>



namespace qvk
{

  swapchain::swapchain(qvk::device &device, qvk::window &window)
  {
    // Assess their abilities
    auto caps = device.phys_dev().getSurfaceCapabilitiesKHR(window.surface());
    auto formats = device.phys_dev().getSurfaceFormatsKHR(window.surface());


    // Format Selection
    vk::SurfaceFormatKHR selected_format = formats[0];
    for (const auto &f: formats)
      if (f.format == vk::Format::eB8G8R8A8Unorm && f.colorSpace == vk::ColorSpaceKHR::eSrgbNonlinear) {
        selected_format = f;
        break;
      }

    m_format = selected_format.format;



    // Resolution Setting
    if (caps.currentExtent.width != std::numeric_limits<u32>::max())
      m_extent = caps.currentExtent;
    else
      m_extent = vk::Extent2D{(u32)window.width(), (u32)window.height()};



    // Create Swapchain
    m_image_count = std::max<u32>(caps.minImageCount, 3);
    if (caps.maxImageCount > 0 && m_image_count > caps.maxImageCount) {
      m_image_count = caps.maxImageCount;
    }

    vk::SwapchainCreateInfoKHR swp_info(
      {}, window.surface(), m_image_count, m_format,
      selected_format.colorSpace, m_extent, 1,
      vk::ImageUsageFlagBits::eColorAttachment
    );
    
    swp_info.setPresentMode(vk::PresentModeKHR::eFifo);
    swp_info.setPreTransform(caps.currentTransform);
    swp_info.setCompositeAlpha(vk::CompositeAlphaFlagBitsKHR::eOpaque);
    //swp_info.setClipping(VK_TRUE);

    m_swapchain = vk::raii::SwapchainKHR(device.vdevice(), swp_info);



    // Access Image Views
    m_images = m_swapchain.getImages();
    for (const auto &img: m_images) {
      vk::ImageViewCreateInfo view_info(
        {}, img, vk::ImageViewType::e2D, m_format,
        {}, {vk::ImageAspectFlagBits::eColor, 0, 1, 0, 1}
      );
      m_image_views.push_back(vk::raii::ImageView(device.vdevice(), view_info));
    }

    // --- DERİNLİK (Z-BUFFER) RESMİ OLUŞTURMA ---
    vk::ImageCreateInfo depth_info(
      {}, vk::ImageType::e2D, m_depth_format,
      vk::Extent3D(m_extent.width, m_extent.height, 1),
      1, 1, vk::SampleCountFlagBits::e1, vk::ImageTiling::eOptimal,
      vk::ImageUsageFlagBits::eDepthStencilAttachment
    );
    m_depth_image = vk::raii::Image(device.vdevice(), depth_info);

    // Bellek tahsisi (VRAM)
    vk::MemoryRequirements mem_reqs = m_depth_image.getMemoryRequirements();
    vk::PhysicalDeviceMemoryProperties mem_props = device.phys_dev().getMemoryProperties();
    u32 mem_type_idx = 0;
    
    for (u32 i = 0; i < mem_props.memoryTypeCount; i++) {
      if ((mem_reqs.memoryTypeBits & (1 << i)) &&
          (mem_props.memoryTypes[i].propertyFlags & vk::MemoryPropertyFlagBits::eDeviceLocal) == vk::MemoryPropertyFlagBits::eDeviceLocal) {
        mem_type_idx = i;
        break;
      }
    }

    vk::MemoryAllocateInfo alloc_info(mem_reqs.size, mem_type_idx);
    m_depth_memory = vk::raii::DeviceMemory(device.vdevice(), alloc_info);
    m_depth_image.bindMemory(*m_depth_memory, 0);

    // Derinlik Görünümü (ImageView)
    vk::ImageViewCreateInfo depth_view_info(
      {}, *m_depth_image, vk::ImageViewType::e2D, m_depth_format,
      {}, {vk::ImageAspectFlagBits::eDepth, 0, 1, 0, 1}
    );
    m_depth_view = vk::raii::ImageView(device.vdevice(), depth_view_info);
  }

}
