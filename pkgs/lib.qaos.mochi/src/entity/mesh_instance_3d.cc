/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#include "mochi/entity/mesh_instance_3d.hh"
#include "mochi/world/components.hh"


namespace mochi::entity
{

  MeshInstance3D::MeshInstance3D(core &eng): Node3D(eng)
  {
    m_core.registry().emplace<RenderableComponent>(m_entity);
  }

  fun MeshInstance3D::update_component() -> void
  {
    auto &rc = m_core.registry().get<RenderableComponent>(m_entity);
    rc.mesh = m_mesh;
    rc.texture = m_texture;
    rc.pipeline = m_pipeline;
  }

  fun MeshInstance3D::set_mesh(sptr<asset::mesh> mesh) -> void
  {
    m_mesh = mesh;
    update_component();
  }

  fun MeshInstance3D::set_material(sptr<rhi::pipeline> pipeline, sptr<asset::texture2> texture) -> void
  {
    m_pipeline = pipeline;
    m_texture = texture;
    update_component();
  }

}
