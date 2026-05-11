/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#pragma once

#include "mochi/basis.hh"
#include "mochi/geometry.hh"
#include "mochi/rhi/buffer.hh"
#include <vulkan/vulkan_raii.hpp>
#include "src/entt/entt.hpp"



namespace mochi::ecs
{

  /** @brief Represents the raw point light data passed to shaders. */
  struct point_light_t {
    vec4<f32> position;
    vec4<f32> color;
  };
  
  /** @brief Buffer info describing the light3d_t memory layout. */
  extern sptr<rhi::info<rhi::buffer>> point_light_i;



  /** @brief Represents a light entity. */
  struct PointLight {
    vec3<f32> color = {1,1,1};
    f32 intensity = 10;
  };

}
