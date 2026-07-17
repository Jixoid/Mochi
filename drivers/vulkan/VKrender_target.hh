/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/

#pragma once

#include "mochi/rhi/render_target.hh"
#include <vulkan/vulkan.hpp>
#include <vulkan/vulkan_raii.hpp>



namespace mochi::rhi::vulkan
{

  struct VK_RenderTarget: RenderTarget {
    public:
      VK_RenderTarget(
        vk::Image     color_image,
        vk::ImageView color_view,
        vk::Format    color_format,

        vk::Image     depth_image,
        vk::ImageView depth_view,
        vk::Format    depth_format,

        vk::Extent2D  extent,

        vk::ImageLayout final_layout
      )
        : color_image(color_image)
        , color_view(color_view)
        , color_format(color_format)

        , depth_image(depth_image)
        , depth_view(depth_view)
        , depth_format(depth_format)

        , extent(extent)

        , final_layout(final_layout)
      {}

    public:
      vk::Image     color_image;
      vk::ImageView color_view;
      vk::Format    color_format;
      
      vk::Image     depth_image;
      vk::ImageView depth_view;
      vk::Format    depth_format;
      
      vk::Extent2D  extent;
      
      vk::ImageLayout final_layout;
  };

}
