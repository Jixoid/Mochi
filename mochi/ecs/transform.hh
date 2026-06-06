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
#include "src/entt/entt.hpp"



namespace mochi::ecs
{

  /** @brief Represents the local and global transform of an entity. */
  struct Transform {
    mat4x3<f32> model = mat4x3<f32>();
  };

}
