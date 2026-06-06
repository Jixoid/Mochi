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
#include <vulkan/vulkan_raii.hpp>
#include "vk_mem_alloc.h"
#include "mochi/rhi/render_target.hh"



namespace mochi::module
{

  struct display
  {
    public:
      explicit display(rhi::device &device, module::memory &memory, std::string_view title, int width, int height);
      ~display();
      
      rhi::device &m_device;
      module::memory &m_memory;


    // window
    private:
      GLFWwindow *m_window{};
      vk::raii::SurfaceKHR vk_surface{nil};

      i32 m_width{}, m_height{};
      bool m_resized{};

    public:
      inline fun  glfw() { return m_window; }
      inline fun& surface() { return vk_surface; }
      inline fun  width()  { return m_width; }
      inline fun  height() { return m_height; }


    private:
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
      /** @brief Rebuild Swapchain based on current window sizes. */
      fun recreate_swapchain() -> void;

      /** @brief Acquire next image index from swapchain */
      fun acquire_next_image(vk::Semaphore image_available_sem) -> u32;

      /** @brief Present image to screen */
      fun present(vk::Semaphore render_finished_sem, u32 image_index) -> void;

      /** @brief Get render target for the given image index */
      fun get_render_target(u32 image_index) -> rhi::render_target;

      /** @brief Get the render finished semaphore for the given image index */
      fun get_render_finished_sem(u32 image_index) -> vk::Semaphore;

    private:      
      vk::raii::SwapchainKHR m_swapchain{nil};
      std::vector<vk::Image> m_images;
      std::vector<vk::raii::ImageView> m_image_views;
      std::vector<vk::raii::Semaphore> m_render_finished_sems;
      vk::Format m_format;
      vk::Extent2D m_extent;
      u32 m_image_count;

      vk::Image m_depth_image{nil};
      VmaAllocation m_depth_allocation{nil};
      vk::raii::ImageView m_depth_view{nil};
      vk::Format m_depth_format{vk::Format::eD32Sfloat};

    public:
      inline fun& get() { return m_swapchain; }
      inline fun& images() { return m_images; }
      inline fun& image_views() { return m_image_views; }
      inline fun& format() { return m_format; }
      inline fun  extent() const { return m_extent; }
      inline fun  image_count() { return m_image_count; }
      inline fun& depth_image() { return m_depth_image; }
      inline fun& depth_view() { return m_depth_view; }
      inline fun  depth_format() const { return m_depth_format; }

  };

}
