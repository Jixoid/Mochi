/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#include "mochi/entity/camera_3d.hh"
#include "mochi/ecs/camera_3d.hh"
#include "mochi/core.hh"



namespace mochi::entity
{

  Camera3D::Camera3D(core &eng): Node3D(eng)
  {
    m_core.registry().emplace<ecs::Camera3D>(m_entity);
    updateComponent();
  }

  fun Camera3D::updateComponent() -> void
  {
    auto& cc = m_core.registry().get<ecs::Camera3D>(m_entity);
    cc.fov = m_fov;
    cc.near = m_near;
    cc.far = m_far;
  }

  fun Camera3D::setFov(f32 fov) -> void
  {
    m_fov = fov;
    updateComponent();
  }

  fun Camera3D::setNear(f32 n) -> void
  {
    m_near = n;
    updateComponent();
  }

  fun Camera3D::setFar(f32 f) -> void
  {
    m_far = f;
    updateComponent();
  }

}
