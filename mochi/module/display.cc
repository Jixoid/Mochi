/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#include "mochi/except.hh"
#include "mochi/module/memory.hh"
#include "mochi/module/display.hh"
#include "mochi/rhi/manager/device_manager.hh"
#include <GLFW/glfw3.h>
#include <string_view>



namespace mochi::module
{

  fun __attribute__((constructor())) glfw_init() { glfwInit(); }
  fun __attribute__((destructor()))  glfw_fini() { glfwTerminate(); }



  display::display(rhi::DeviceManager &device, module::memory &memory, std::string_view title, int width, int height)
    : m_device(device)
    , m_memory(memory)
  {
    glfwWindowHint(GLFW_CLIENT_API, GLFW_NO_API);

    m_window = glfwCreateWindow(
      width, height,
      std::string(title).c_str(),
      nil, nil
    );

    glfwGetFramebufferSize(m_window, &m_width, &m_height);

    glfwSetWindowUserPointer(m_window, this);

    glfwSetFramebufferSizeCallback(m_window, framebuffer_resize_callback);

    m_swapchain_mgr = rhi::SwapchainManager::make(m_device);
    m_swapchain_mgr->init(m_window, m_width, m_height);
  }

  display::~display()
  {
  }



  fun display::recreate_swapchain() -> void
  {
    // Wait if window is minimized
    int width = 0, height = 0;
    glfwGetFramebufferSize(m_window, &width, &height);
    while (width == 0 || height == 0) {
      glfwGetFramebufferSize(m_window, &width, &height);
      glfwWaitEvents();
    }
    
    m_swapchain_mgr->resize(width, height);
    m_resized = false;
  }


  void display::framebuffer_resize_callback(GLFWwindow* win, int width, int height)
  {
    auto app = reinterpret_cast<display*>(glfwGetWindowUserPointer(win));
    
    app->m_resized = true;
    app->m_width = width;
    app->m_height = height;
  }

  fun display::proc_events() -> bool
  {
    glfwPollEvents();

    if (m_width == 0 || m_height == 0) {
      glfwWaitEvents();
      return !glfwWindowShouldClose(m_window);
    }

    return !glfwWindowShouldClose(m_window);
  }

  fun display::acquire_next_image(void* image_available_sem) -> u32
  {
    return m_swapchain_mgr->acquireNextImage(image_available_sem);
  }

  fun display::present(void* render_finished_sem, u32 image_index) -> void
  {
    m_swapchain_mgr->present(image_index, render_finished_sem);
  }

  fun display::get_render_target(u32 image_index) -> rhi::render_target
  {
    return m_swapchain_mgr->getRenderTarget(image_index);
  }
  
}
