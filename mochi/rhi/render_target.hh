/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#pragma once

#include "mochi/basis.hh"
#include "vulkan/vulkan.hpp"
#include <vulkan/vulkan_raii.hpp>



namespace mochi::rhi
{

  struct render_target
  {
    vk::Image     color_image{nil};
    vk::ImageView color_view{nil};
    vk::Format    color_format{};
    
    vk::Image     depth_image{nil};
    vk::ImageView depth_view{nil};
    vk::Format    depth_format{};

    vk::Extent2D  extent{};
    
    vk::ImageLayout final_layout{vk::ImageLayout::eShaderReadOnlyOptimal};
  };

}
