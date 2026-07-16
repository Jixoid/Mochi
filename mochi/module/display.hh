/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#pragma once

#include "mochi/types.hh"
#include <GLFW/glfw3.h>
#include <string_view>
#include "mochi/math/extent.hh"
#include "mochi/rhi/manager/swapchain_manager.hh"
#include "vk_mem_alloc.h"
#include "mochi/rhi/render_target.hh"



namespace mochi::module
{

  struct display
  {
    public:
      explicit display(rhi::DeviceManager &device, module::memory &memory, std::string_view title, int width, int height);
      ~display();
      
      rhi::DeviceManager &m_device;
      module::memory &m_memory;


    // window
    private:
      GLFWwindow *m_window{};
      sptr<rhi::SwapchainManager> m_swapchain_mgr;
      
    public:
      inline fun glfw() -> GLFWwindow* { return m_window; }
      
    public:
      inline fun  width()  { return m_width; }
      inline fun  height() { return m_height; }
      inline fun extent() -> extent<2, u32> { return {(u32)m_width, (u32)m_height}; }

    private:
      int m_width{};
      int m_height{};
      bool m_resized{false};
      
      static void framebuffer_resize_callback(GLFWwindow* win, int width, int height);
    
    public:
      static inline fun extensions() -> std::vector<const char*> {
        u32 glfwExtensionCount{};
        const char **glfwExtensions = glfwGetRequiredInstanceExtensions(&glfwExtensionCount);
        return std::vector<const char*>(glfwExtensions, glfwExtensions + glfwExtensionCount);
      }

      /**
       * @brief Process window events.
       * @return True if the window should stay open, false if it should close.
       */
      fun proc_events() -> bool;

    inline fun resized() { return m_resized; }



    // swapchain
    public:
      fun recreate_swapchain() -> void;
      fun acquire_next_image(void* image_available_sem) -> u32;
      fun present(void* render_finished_sem, u32 image_index) -> void;

      fun get_render_target(u32 image_index) -> rhi::render_target;
      
      inline fun swapchain_mgr() -> rhi::SwapchainManager& { return *m_swapchain_mgr; }
  };
  
}
