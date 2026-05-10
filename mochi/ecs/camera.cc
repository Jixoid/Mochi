/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#include "mochi/ecs/camera.hh"
#include "mochi/basis.hh"
#include "mochi/geometry.hh"
#include "mochi/rhi/buffer.hh"
#include <vulkan/vulkan_raii.hpp>
#include "src/entt/entt.hpp"



namespace mochi::ecs
{

  rhi::info<rhi::buffer> camera_i(
    sizeof(camera_t),
    vt::make_list<
      mat4<f32>, // View
      mat4<f32>  // Projection
    >()
  );

}
