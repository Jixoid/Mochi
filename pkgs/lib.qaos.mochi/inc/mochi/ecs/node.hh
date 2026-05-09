/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#pragma once

#include <vulkan/vulkan_raii.hpp>
#include "src/entt/entt.hpp"



namespace mochi::ecs
{

  /** @brief Represents the hierarchy relationship of an entity. */
  struct Node {
    entt::entity parent = entt::null;
    std::vector<entt::entity> children;
  };

}
