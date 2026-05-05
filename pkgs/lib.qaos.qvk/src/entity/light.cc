/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#include "qvk/entity/node.hh"
#include "qvk/entity/light.hh"
#include "qvk/entity/buffer.hh"
#include "qvk/geometry.hh"
#include "qvk/core.hh"
#include "qvk/types.hh"
#include <vulkan/vulkan_raii.hpp>
#include <vulkan/vulkan.h>



namespace qvk
{

  info<buffer> light_i = info<buffer>(
    sizeof(light_t),
    gt::make_list<
      vec4<f32>, // position
      vec4<f32>  // color
    >()
  );

  
  light::light(core &core, node *parent, mat4<f32> model, vec3<f32> color, f32 intensity)
    : node(parent, model)
    , m_memory(core.sub<memory>())
    , m_color(color)
    , m_intensity(intensity)
  {
    m_memory.push(this);
  }

  light::~light()
  {
    m_memory.pop(this);
  }
  

}
