/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/

#pragma once

#include "mochi/basis.hh"
#include "mochi/math/mat4x3.hh"
#include <vulkan/vulkan_raii.hpp>
#include "entt/entt.hpp"



namespace mochi::ecs
{
  
  struct Skeleton {
    std::vector<mat4x3<f32>> bones;
  };

}
