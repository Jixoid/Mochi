/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#include "mochi/entity/camera_3d.hh"
#include "mochi/world/components.hh"



namespace mochi::entity
{

  Camera3D::Camera3D(core &eng): Node3D(eng)
  {
    m_core.registry().emplace<CameraComponent>(m_entity);
    update_component();
  }

  fun Camera3D::update_component() -> void
  {
    auto& cc = m_core.registry().get<CameraComponent>(m_entity);
    cc.fov = m_fov;
    cc.near = m_near;
    cc.far = m_far;
    
    // Note: The main loop typically updates view and projection matrices
    // using this camera's parameters.
  }

  fun Camera3D::set_fov(f32 fov) -> void
  {
    m_fov = fov;
    update_component();
  }

  fun Camera3D::set_near(f32 n) -> void
  {
    m_near = n;
    update_component();
  }

  fun Camera3D::set_far(f32 f) -> void
  {
    m_far = f;
    update_component();
  }

  fun Camera3D::make_current() -> void
  {
    // Implementation can be added if the engine tracks a "current" camera
  }

}
