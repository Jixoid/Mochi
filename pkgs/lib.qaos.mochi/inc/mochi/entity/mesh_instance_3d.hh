/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#pragma once

#include "mochi/entity/node3d.hh"
#include "mochi/asset/mesh.hh"
#include "mochi/asset/texture.hh"
#include "mochi/rhi/pipeline.hh"



namespace mochi::entity
{

  /** @brief Renderable 3D object. Wraps RenderableComponent. */
  class MeshInstance3D: public Node3D
  {
    public:
      explicit MeshInstance3D(core &eng);
      virtual ~MeshInstance3D() override = default;


    private:
      sptr<asset::mesh> m_mesh;
      sptr<asset::texture2> m_texture;
      sptr<rhi::pipeline> m_pipeline;
      
      fun update_component() -> void;

    public:
      inline fun get_mesh() const { return m_mesh; }
      inline fun get_texture() const { return m_texture; }
      inline fun get_pipeline() const { return m_pipeline; }

      fun set_mesh(sptr<asset::mesh> mesh) -> void;
      fun set_material(sptr<rhi::pipeline> pipeline, sptr<asset::texture2> texture) -> void;
  };

}
