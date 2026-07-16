/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#pragma once

#include "mochi/basis.hh"
#include "mochi/math/math.hh"
#include <vulkan/vulkan_raii.hpp>
#include "entt/entt.hpp"



namespace mochi::ecs
{

  struct point_light_t {
    vec4<f32> position;
    vec4<f32> color;
  };
  

  struct PointLight {
    vec3<f32> color = {1,1,1};
    f32 intensity = 10;
  };

}
