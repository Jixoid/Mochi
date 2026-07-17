/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#include "mochi/asset/material.hh"
#include "mochi/asset/texture.hh"
#include "mochi/except.hh"
#include "mochi/manager/material_manager.hh"
#include "mochi/rhi/render_target.hh"
#include <variant>



namespace mochi::asset
{

  Material::Material(Core &core): m_core(core) {}


  
  fun Material::desc(rhi::RenderTarget &target) -> sptr<manager::MaterialDesc> {
    auto Albedo = [&](){
      if (std::holds_alternative<vec3<f32>>(m_albedo)) return manager::MaterialAlbedo::Color;
      ef (std::holds_alternative<sptr<asset::Texture2>>(m_albedo)) return manager::MaterialAlbedo::Texture;
      else
        throw except("unknown type");
    };


    manager::MaterialProps props = {
      .method = manager::MaterialMethod::PBR,
      .albedo = Albedo(),
      .count  = m_count,
      .polymode = m_polymode,
      .primitiveTopology = m_primitiveTopology,
    };

    return std::move(m_core.sub<manager::MaterialManager>().getMaterialDesc(target, props));
  }

}
