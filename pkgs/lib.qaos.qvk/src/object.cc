/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#include "Basis.hh"
#include "qvk/object.hh"
#include "qvk/memory.hh"
#include "qvk/pipeline.hh"
#include "qvk/engine.hh"
#include <vulkan/vulkan_raii.hpp>



namespace qvk
{

  object::object(qvk::mat4<f32> model, qvk::camera *camera, qvk::pipeline *pipeline, qvk::buffer *buffer, u32 count)
    : m_model(model)
    , m_camera(camera)
    , m_pipeline(pipeline)
    , m_buffer(buffer)
    , m_count(count)
  {}



  fun object::make(qvk::engine &engine, qvk::mat4<f32> model,  qvk::camera *camera, qvk::pipeline *pipeline, qvk::buffer *buffer, u32 count) -> object*
  {
    auto obj = new object(model, camera, pipeline, buffer, count);

    engine.sub<memory>().push<object>(obj);
    return obj;
  }

}
