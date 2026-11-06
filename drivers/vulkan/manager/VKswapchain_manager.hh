/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/

#pragma once

#include "drivers/vulkan/VKrender_target.hh"
#include "mochi/basis.hh"
#include "mochi/rhi/manager/swapchain_manager.hh"
#include <vulkan/vulkan_raii.hpp>
#include "mochi/rhi/render_target.hh"
#include "vk_mem_alloc.h"



namespace mochi::rhi::vulkan
{

  struct VK_SwapchainManager final: public rhi::SwapchainManager {
    public:
      explicit VK_SwapchainManager(rhi::Device &dmng);
      ~VK_SwapchainManager() override;

      VkSurfaceKHR m_surface{VK_NULL_HANDLE};
      bool m_owns_surface{false};
      vk::raii::SwapchainKHR m_swapchain{nil};
      std::vector<vk::Image> m_images;
      std::vector<vk::raii::ImageView> m_image_views;
      std::vector<vk::raii::Semaphore> m_render_finished_sems;
      vk::raii::Fence m_acquire_fence{nil};
      vk::raii::CommandPool m_cmd_pool{nil};
      vk::raii::CommandBuffer m_cmd_buffer{nil};
      
      vk::raii::Image m_depth_image_handle{nil};
      vk::raii::DeviceMemory m_depth_memory{nil};
      vk::raii::ImageView m_depth_view{nil};
      
      std::vector<VK_RenderTarget> m_targets;
      u32 m_width{}, m_height{};
      vk::Format m_format{};
      vk::Format m_depth_format{vk::Format::eD32Sfloat};
      vk::Extent2D m_extent{};

      void create_resources();
      void destroy_resources();

    public:
      fun init(void* windowHandle, u32 width, u32 height) -> void override;
      fun initWithSurface(VkSurfaceKHR surface, u32 width, u32 height) -> void;
      fun resize(u32 width, u32 height) -> void override;

      fun acquireNextImage(void* signalSemaphore) -> u32 override;
      fun present(u32 imageIndex, void* waitSemaphore) -> void override;

      fun getRenderTarget(u32 index) -> RenderTarget& override;
      fun getRenderTargetCount() const -> u32 override;

      fun getRenderFinishedSemaphore(u32 image_index) -> void* override;
  };
  
}
