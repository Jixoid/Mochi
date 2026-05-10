/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#pragma once

#include "mochi/basis.hh"
#include "mochi/types.hh"
#include "mochi/rhi/render_target.hh"
#include "mochi/module/device.hh"
#include <span>
#include <vulkan/vulkan_raii.hpp>



namespace mochi::module
{

  /** @brief Manages the rendering loop, command buffers, and frame synchronization. */
  struct renderer
  {
    public:
      /** @brief Maximum number of frames that can be processed concurrently. */
      static constexpr u32 MAX_FRAMES_IN_FLIGHT = 2;

    public:
      /**
       * @brief Initialize a new renderer.
       * @param device The logical device.
       */
      explicit renderer(module::device &device);


    private:
      module::device  &m_device;

      vk::raii::CommandPool                m_cmd_pool;
      std::vector<vk::raii::CommandBuffer> m_cmd_buffers;

      // Synchronization Objects (per frame)
      std::vector<vk::raii::Semaphore> m_image_available_sems;
      std::vector<vk::raii::Fence>     m_in_flight_fences;

      u32 m_current_frame{}; // Current frame in flight (e.g., 0 or 1)

    public:
      /** @brief Access the Vulkan RAII command pool. */
      inline fun& cmd_pool() { return m_cmd_pool; }
      /** @brief Get the current frame in flight index. */
      inline fun current_frame() const { return m_current_frame; }

      inline fun& get_image_available_sem(u32 frame) { return *m_image_available_sems[frame]; }

    public:
      /**
       * @brief Begin a rendering pass on the given render target.
       * @param cmd The command buffer to record into.
       * @param target The render target (framebuffer/texture)
       * @param clear_color The color used to clear the screen (RGBA).
       */
      fun begin_pass(vk::raii::CommandBuffer &cmd, const rhi::render_target &target, const std::array<float, 4> &clear_color) -> void;
      
      /**
       * @brief End the rendering pass on the given render target.
       * @param cmd The command buffer being recorded.
       * @param target The render target to transition to its final layout.
       */
      fun end_pass(vk::raii::CommandBuffer &cmd, const rhi::render_target &target) -> void;


      /**
       * @brief Start a new frame, setting up synchronization.
       * @return Reference to the active command buffer for the frame.
       */
      fun begin_frame() -> vk::raii::CommandBuffer&;
      
      /**
       * @brief Submit the command buffer to the GPU.
       * @param cmd The active command buffer to submit.
       * @param wait_sems Semaphores to wait on before execution.
       * @param signal_sems Semaphores to signal after execution finishes.
       */
      fun end_frame(vk::raii::CommandBuffer &cmd, std::span<vk::Semaphore> wait_sems, std::span<vk::Semaphore> signal_sems) -> void;
  };
  
}
