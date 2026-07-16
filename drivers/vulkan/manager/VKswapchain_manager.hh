/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/

#pragma once

#include "mochi/basis.hh"
#include "mochi/rhi/manager/swapchain_manager.hh"
#include "drivers/vulkan/manager/VKdevice_manager.hh"
#include <vulkan/vulkan_raii.hpp>
#include "vk_mem_alloc.h"

namespace mochi::rhi::vulkan
{
  struct VK_SwapchainManager final : public rhi::SwapchainManager {
    public:
      explicit VK_SwapchainManager(rhi::DeviceManager &dmng);
      ~VK_SwapchainManager() override;

    private:
      vk::raii::SurfaceKHR m_surface{nil};
      vk::raii::SwapchainKHR m_swapchain{nil};
      std::vector<vk::Image> m_images;
      std::vector<vk::raii::ImageView> m_image_views;
      
      vk::Image m_depth_image{nil};
      VmaAllocation m_depth_allocation{nil};
      vk::raii::ImageView m_depth_view{nil};
      
      std::vector<render_target> m_targets;
      u32 m_width{}, m_height{};
      vk::Format m_format{};
      vk::Format m_depth_format{vk::Format::eD32Sfloat};
      vk::Extent2D m_extent{};

      void create_resources();
      void destroy_resources();

    public:
      fun init(void* windowHandle, u32 width, u32 height) -> void override;
      fun resize(u32 width, u32 height) -> void override;

      fun acquireNextImage(void* signalSemaphore) -> u32 override;
      fun present(u32 imageIndex, void* waitSemaphore) -> void override;

      fun getRenderTarget(u32 index) -> render_target& override;
      fun getRenderTargetCount() const -> u32 override;
  };
}
