/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#include "mochi/entity/mesh_instance_3d.hh"
#include "mochi/ecs/mesh_instance_3d.hh"
#include "mochi/core.hh"



namespace mochi::entity
{

  MeshInstance3D::MeshInstance3D(core &eng): Node3D(eng)
  {
    m_core.registry().emplace<ecs::MeshInstance3D>(m_entity);
  }

  fun MeshInstance3D::updateComponent() -> void
  {
    auto &rc = m_core.registry().get<ecs::MeshInstance3D>(m_entity);
    rc.mesh = m_mesh;
    rc.texture = m_texture;
    rc.pipeline = m_pipeline;
  }

  fun MeshInstance3D::setMesh(sptr<asset::mesh> mesh) -> void
  {
    m_mesh = mesh;
    updateComponent();
  }

  fun MeshInstance3D::setMaterial(sptr<rhi::pipeline> pipeline, sptr<asset::texture2> texture) -> void
  {
    m_pipeline = pipeline;
    m_texture = texture;
    updateComponent();
  }

}
