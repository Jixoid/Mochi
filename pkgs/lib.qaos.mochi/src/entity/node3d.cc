/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#include "mochi/entity/node3d.hh"
#include "mochi/world/components.hh"



namespace mochi::entity
{

  Node3D::Node3D(core &eng): Node(eng)
  {
    m_core.registry().emplace<TransformComponent>(m_entity);
    update_transform();
  }

  fun Node3D::update_transform() -> void
  {
    auto& t = m_core.registry().get<TransformComponent>(m_entity);
    t.model = mat4<f32>::model(m_position, m_rotation, m_scale);
  }

  fun Node3D::set_position(const vec3<f32>& pos) -> void
  {
    m_position = pos;
    update_transform();
  }

  fun Node3D::set_rotation(const quaternion<f32>& rot) -> void
  {
    m_rotation = rot;
    update_transform();
  }

  fun Node3D::set_scale(const vec3<f32>& scale) -> void
  {
    m_scale = scale;
    update_transform();
  }

}
