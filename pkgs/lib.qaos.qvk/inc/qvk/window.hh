/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#pragma once

#include "qvk/types.hh"
#include "Basis.hh"
#include <string_view>
#include <vulkan/vulkan_raii.hpp>
#include <vulkan/vulkan.h>
#define GLFW_INCLUDE_VULKAN
#include <GLFW/glfw3.h>



namespace qvk
{

  struct window
  {
    public:
      explicit window(qvk::bridge &bridge, std::string_view title, int width, int height);


    private:
      GLFWwindow *m_window{};
      vk::raii::SurfaceKHR vk_surface;

      i32 m_width{}, m_height{};
      bool m_resized{};


    public:
      inline fun& surface() { return vk_surface; }
      inline fun  width()  { return m_width; }
      inline fun  height() { return m_height; }

      
    public:
      static inline fun extensions() -> std::vector<const char*> {
        u32 glfwExtensionCount{};
        const char **glfwExtensions = glfwGetRequiredInstanceExtensions(&glfwExtensionCount);
        return std::vector<const char*>(glfwExtensions, glfwExtensions + glfwExtensionCount);
      }


    private:
      static void framebuffer_resize_callback(GLFWwindow* win, int width, int height);

    public:
      fun proc_events() -> bool;

  };

}
