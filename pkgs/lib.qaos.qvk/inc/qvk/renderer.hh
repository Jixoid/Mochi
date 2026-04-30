/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#pragma once

#include "Basis.hh"
#include "qvk/types.hh"
#include "qvk/device.hh"
#include "qvk/swapchain.hh"
#include <vector>
#include <vulkan/vulkan_raii.hpp>



namespace qvk
{

  struct renderer
  {
    public:
      static constexpr u32 MAX_FRAMES_IN_FLIGHT = 2;

    public:
      explicit renderer(device &device, swapchain &swapchain);


    private:
      device &m_device;
      swapchain &m_swapchain;

      vk::raii::CommandPool                m_cmd_pool;
      std::vector<vk::raii::CommandBuffer> m_cmd_buffers;

      // Senkronizasyon Nesneleri (Her kare için ayrı)
      std::vector<vk::raii::Semaphore> m_image_available_sems;
      std::vector<vk::raii::Semaphore> m_render_finished_sems;
      std::vector<vk::raii::Fence>     m_in_flight_fences;

      u32 m_current_frame{}; // Hangi karedeyiz? (0 veya 1)
      u32 m_image_index{};   // Swapchain'den alınan resmin indisi

    public:
      inline fun& cmd_pool() { return m_cmd_pool; }
      inline fun image_index() const { return m_image_index; }
      inline fun current_frame() const { return m_current_frame; }

    public:
      fun begin_swapchain_rendering(vk::raii::CommandBuffer &cmd, const std::array<float, 4> &clear_color) -> void;
      fun end_swapchain_rendering(vk::raii::CommandBuffer &cmd) -> void;


      fun begin_frame() -> vk::raii::CommandBuffer&;
      fun end_frame(vk::raii::CommandBuffer &cmd) -> void;
  };
  
}
