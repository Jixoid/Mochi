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
#include "entt/entity/entity.hpp"



namespace mochi::ecs
{

  struct Mesh {
    sptr<asset::Mesh> mesh;
    entt::entity skeleton = entt::null;
  };

}
