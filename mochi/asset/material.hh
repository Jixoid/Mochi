/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#pragma once

#include "mochi/basis.hh"
#include "mochi/rhi/manager/material_manager.hh"
#include "mochi/core.hh"
#include <variant>




namespace mochi::asset
{

  struct Material {
    public:
      explicit Material(core &core);

    private:
      core &m_core;
      std::variant<vec3<f32>, sptr<asset::texture2>> m_albedo;
      rhi::PolygonMode m_polymode{rhi::PolygonMode::Fill};
      rhi::PrimitiveTopology m_primitiveTopology{rhi::PrimitiveTopology::TriangleList};
      rhi::MaterialCount m_count{rhi::MaterialCount::Single};

    public:
      fun is_color() { return std::holds_alternative<vec3<f32>>(m_albedo); }
      fun color() { return std::get<vec3<f32>>(m_albedo); }
      fun setColor(vec3<f32> val) { m_albedo = val; }
      
      fun is_texture() { return std::holds_alternative<sptr<asset::texture2>>(m_albedo); }
      fun texture() { return std::get<sptr<asset::texture2>>(m_albedo); }
      fun setTexture(sptr<asset::texture2> val) { m_albedo = val; }

      fun polymode() { return m_polymode; }
      fun setPolymode(rhi::PolygonMode val) { m_polymode = val; }

      fun primitiveTopology() { return m_primitiveTopology; }
      fun setPrimitiveTopology(rhi::PrimitiveTopology val) { m_primitiveTopology = val; }

      fun count() { return m_count; }
      fun setCount(rhi::MaterialCount val) { m_count = val; }

      fun desc(rhi::render_target &target) -> sptr<rhi::MaterialDesc>;
  };

}
