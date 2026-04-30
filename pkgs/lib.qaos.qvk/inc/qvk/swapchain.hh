/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#pragma once

#include "Basis.hh"
#include "types.hh"
#include <vulkan/vulkan_raii.hpp>
#include <vulkan/vulkan.h>



namespace qvk
{

  struct swapchain
  {
    public:
      explicit swapchain(qvk::device &device, qvk::window &window);


    private:
      vk::raii::SwapchainKHR m_swapchain{nullptr};
      std::vector<vk::Image> m_images;
      std::vector<vk::raii::ImageView> m_image_views;
      vk::Format m_format;
      vk::Extent2D m_extent;
      u32 m_image_count;

    public:
      inline fun& get() { return m_swapchain; }
      inline fun& images() { return m_images; }
      inline fun& image_views() { return m_image_views; }
      inline fun& format() { return m_format; }
      inline fun  extent() const { return m_extent; }
      inline fun  image_count() { return m_image_count; }
  };

}
