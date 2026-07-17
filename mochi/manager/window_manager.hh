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
#include <GLFW/glfw3.h>
#include <string_view>
#include "mochi/math/extent.hh"
#include "mochi/rhi/manager/swapchain_manager.hh"
#include "vk_mem_alloc.h"
#include "mochi/rhi/render_target.hh"



namespace mochi::manager
{

  struct WindowManager: noncopy {
    public:
      explicit WindowManager(rhi::DeviceManager &dmng, manager::SceneManager &smng, std::string_view title, int width, int height);
      ~WindowManager();
      
      rhi::DeviceManager &m_dmng;
      manager::SceneManager &m_smng;


    // window
    private:
      GLFWwindow *m_window{};
      sptr<rhi::SwapchainManager> m_swapchain_mgr;
      
    public:
      fun glfw() -> GLFWwindow* { return m_window; }
      
    public:
      fun  width()  { return m_width; }
      fun  height() { return m_height; }
      fun extent() -> extent<2, u32> { return {(u32)m_width, (u32)m_height}; }

    private:
      int m_width{};
      int m_height{};
      bool m_resized{false};
      
      static void framebuffer_resize_callback(GLFWwindow* win, int width, int height);
    
    public:
      static fun extensions() -> std::vector<const char*> {
        u32 glfwExtensionCount{};
        const char **glfwExtensions = glfwGetRequiredInstanceExtensions(&glfwExtensionCount);
        return std::vector<const char*>(glfwExtensions, glfwExtensions + glfwExtensionCount);
      }

      /// @brief Process window events.
      fun proc_events() -> bool;

      fun resized() { return m_resized; }


    // swapchain
    public:
      fun recreate_swapchain() -> void;
      fun acquire_next_image(void* image_available_sem) -> u32;
      fun present(void* render_finished_sem, u32 image_index) -> void;

      fun get_render_target(u32 image_index) -> rhi::RenderTarget&;
      fun getRenderFinishedSemaphore(u32 image_index) -> void*;
      
      fun swapchain_mgr() -> rhi::SwapchainManager& { return *m_swapchain_mgr; }
  };
  
}
