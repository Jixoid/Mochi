/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#include "Basis.hh"
#include "qvk/geometry.hh"
#include "qvk/types.hh"
#include "qvk/window.hh"
#include "qvk/camera.hh"
#include "qvk/engine.hh"
#include <vulkan/vulkan_raii.hpp>
#include <vulkan/vulkan.h>



namespace qvk
{

  camera::camera(qvk::engine &engine, vec3<f32> pos, vec3<f32> target, vec3<f32> up, f32 fov, f32 near, f32 far)
    : m_window(engine.sub<window>()), m_pos(pos)
    , m_fov(fov), m_near(near), m_far(far)
    , m_rot(quaternion<f32>::lookAt(pos, target, up))
  {
    recalc();
  }

  camera::camera(qvk::engine &engine, vec3<f32> pos, quaternion<f32> rot, f32 fov, f32 near, f32 far)
    : m_window(engine.sub<window>()), m_pos(pos)
    , m_fov(fov), m_near(near), m_far(far)
    , m_rot(rot)
  {
    recalc();
  }



  fun camera::make(qvk::engine &engine, vec3<f32> pos, vec3<f32> target, vec3<f32> up, f32 fov, f32 near, f32 far, metaobj<camera> *meta) -> camera*
  {
    auto obj = new camera(engine, pos, target, up, fov, near, far);

    engine.sub<memory>().push<camera>(obj);
    if (meta) meta->connect(obj);
    return obj;
  }

  fun camera::make(qvk::engine &engine, vec3<f32> pos, quaternion<f32> rot, f32 fov, f32 near, f32 far, metaobj<camera> *meta) -> camera*
  {
    auto obj = new camera(engine, pos, rot, fov, near, far);

    engine.sub<memory>().push<camera>(obj);
    if (meta) meta->connect(obj);
    return obj;
  }




  fun camera::recalc() -> void
  {
    m_proj = mat4<f32>::perspective(
      m_fov,
      (f32)m_window.width()/(f32)m_window.height(),
      m_near,
      m_far
    );

    m_view = mat4<f32>::lookAt(
      m_pos,
      m_pos +m_rot.rotate({0,0,-1}),
      m_rot.rotate({0,1,0})
    );
  }

}
