/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#pragma once

#include "basis.hh"
#include "mochi/geometry.hh"
#include "mochi/rhi/buffer.hh"
#include <vulkan/vulkan_raii.hpp>
#include "src/entt/entt.hpp"



namespace mochi::ecs
{

  /** @brief Represents the raw camera data passed to shaders (View and Projection matrices). */
  struct camera3d_t {
    mat4<f32> view;
    mat4<f32> proj;
  };
  
  /** @brief Buffer info describing the camera3d_t memory layout. */
  extern rhi::info<rhi::buffer> camera3d_i;



  /** @brief Represents a camera entity. */
  struct Camera3D {
    f32 fov = 60;
    f32 near = 0.1;
    f32 far = 1000;
    
    mat4<f32> view = mat4<f32>();
    mat4<f32> proj = mat4<f32>();
  };

}
