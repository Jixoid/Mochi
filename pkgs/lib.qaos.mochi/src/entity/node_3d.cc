/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#include "mochi/entity/node_3d.hh"
#include "mochi/ecs/node_3d.hh"
#include "mochi/core.hh"



namespace mochi::entity
{

  Node3D::Node3D(core &eng): Node(eng)
  {
    m_core.registry().emplace<ecs::Node3D>(m_entity);
    updateTransform();
  }

  fun Node3D::updateTransform() -> void
  {
    auto& t = m_core.registry().get<ecs::Node3D>(m_entity);
    t.model = mat4<f32>::model(m_position, m_rotation, m_scale);
  }

  fun Node3D::setPosition(const vec3<f32>& pos) -> void
  {
    m_position = pos;
    updateTransform();
  }

  fun Node3D::setRotation(const quaternion<f32>& rot) -> void
  {
    m_rotation = rot;
    updateTransform();
  }

  fun Node3D::setScale(const vec3<f32>& scale) -> void
  {
    m_scale = scale;
    updateTransform();
  }

}
