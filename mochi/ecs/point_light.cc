/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#include "mochi/ecs/point_light.hh"
#include "mochi/basis.hh"
#include "mochi/geometry.hh"
#include "mochi/rhi/buffer.hh"
#include <vulkan/vulkan_raii.hpp>
#include "src/entt/entt.hpp"



namespace mochi::ecs
{

  rhi::info<rhi::buffer> point_light_i(
    sizeof(point_light_t),
    vt::make_list<
      vec4<f32>, // Position
      vec4<f32>  // Color
    >()
  );

}
