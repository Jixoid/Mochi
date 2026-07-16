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
#include "mochi/rhi/buffer.hh"
#include "entt/entt.hpp"



namespace mochi::ecs
{
  
  struct instance_data_t {
    vec4<f32> pos_radius;  // xyz: Pozisyon, w: Yarıçap
  };

  struct MultiMesh {
    sptr<asset::Mesh> mesh;
    sptr<rhi::Buffer> instances;
    u32 active_count{0};
  };

}
