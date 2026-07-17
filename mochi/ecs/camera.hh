/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/

#pragma once

#include "mochi/basis.hh"
#include <vulkan/vulkan_raii.hpp>
#include "entt/entt.hpp"
#include "mochi/math/mat4x4.hh"



namespace mochi::ecs
{

  struct camera_t {
    mat4x4<f32> view;
    mat4x4<f32> proj;
  };


  struct Camera {
    f32 fov = 60;
    f32 near = 0.1;
    f32 far = 1000;
    
    mat4x4<f32> view = mat4x4<f32>();
    mat4x4<f32> proj = mat4x4<f32>();
  };

}
