/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#pragma once

#include "mochi/basis.hh"
#include "mochi/asset/mesh.hh"
#include "mochi/asset/material.hh"
#include <vulkan/vulkan_raii.hpp>
#include "src/entt/entt.hpp"



namespace mochi::ecs
{

  /** @brief Represents a renderable entity with geometry and material data. */
  struct Mesh {
    sptr<asset::mesh> mesh;
    sptr<asset::material> material;
    entt::entity skeleton = entt::null;
  };

}
