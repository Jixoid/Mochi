/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#include "mochi/asset/material.hh"
#include "mochi/except.hh"
#include "mochi/module/resource.hh"
#include <variant>



namespace mochi::asset
{

  material::material(core &core)
    : m_core(core)
  {}


  
  fun material::desc(rhi::render_target &target) -> sptr<module::material_desc>
  {
    auto Albedo = [&](){
      if (std::holds_alternative<vec3<f32>>(m_albedo)) return module::material_albedo::maColor;
      ef (std::holds_alternative<sptr<asset::texture2>>(m_albedo)) return module::material_albedo::maTexture;
      else
        throw except("unknown type");
    };


    module::material_props props = {
      .method = module::material_method::mmPBR,
      .albedo = Albedo(),
      .texture = is_texture() ? texture().get() : nil,
    };

    return std::move(m_core.sub<module::resource>().get_or_new_material_desc(target, props));
  }

}
