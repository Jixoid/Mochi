/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#include "Basis.hh"
#include "qvk/world/camera.hh"
#include "qvk/world/node.hh"
#include "qvk/module/memory.hh"
#include "qvk/module/window.hh"
#include "qvk/geometry.hh"
#include "qvk/types.hh"
#include "qvk/core.hh"
#include <vulkan/vulkan_raii.hpp>
#include <vulkan/vulkan.h>



namespace qvk
{

  info<buffer> camera_i = info<buffer>(
    sizeof(camera_t),
    vt::make_list<
      mat4<f32>, // view
      mat4<f32>  // proj
    >()
  );



  camera::camera(core &core, node *parent, mat4<f32> model, f32 fov, f32 near, f32 far)
    : node(parent, model)
    , m_memory(core.sub<module::memory>())
    , m_window(core.sub<module::window>())
    , m_fov(fov), m_near(near), m_far(far)
  {
    m_memory.push(this);

    recalc();
  }

  camera::~camera()
  {
    m_memory.pop(this);
  }



  fun camera::recalc() -> void
  {
    m_view = getModel().inverse();

    m_proj = mat4<f32>::perspective(
      m_fov,
      (f32)m_window.width()/(f32)m_window.height(),
      m_near,
      m_far
    );

    m_memory.sync_camera(this);
  }

}
