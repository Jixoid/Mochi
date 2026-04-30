/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#include "qvk/device.hh"
#include "qvk/window.hh"
#include "qvk/swapchain.hh"
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
  }

}
