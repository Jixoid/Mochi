/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#pragma once

#include "mochi/basis.hh"
#include "mochi/module/resource.hh"
#include "mochi/core.hh"
#include "mochi/rhi/rhi.hh"
#include <variant>
#include <vulkan/vulkan_raii.hpp>



namespace mochi::asset
{

  /** @brief Represents a material that associates a pipeline with a specific descriptor set. */
  struct material
  {
    public:
      /**
       * @brief Construct a new material3d instance.
       * @param core The mochi core instance.
       */
      explicit material(core &core);


    private:
      core &m_core;
      std::variant<vec3<f32>, sptr<asset::texture2>> m_albedo;
      rhi::PolygonMode m_polymode{rhi::PolygonMode::Fill};
      rhi::PrimitiveTopology m_primitiveTopology{rhi::PrimitiveTopology::eTriangleList};

    public:
      inline fun is_color() { return std::holds_alternative<vec3<f32>>(m_albedo); }
      inline fun color() { return std::get<vec3<f32>>(m_albedo); }
      inline fun setColor(vec3<f32> val) { m_albedo = val; }
      
      inline fun is_texture() { return std::holds_alternative<sptr<asset::texture2>>(m_albedo); }
      inline fun texture() { return std::get<sptr<asset::texture2>>(m_albedo); }
      inline fun setTexture(sptr<asset::texture2> val) { m_albedo = val; }

      inline fun polymode() { return m_polymode; }
      inline fun setPolymode(rhi::PolygonMode val) { m_polymode = val; }

      inline fun primitiveTopology() { return m_primitiveTopology; }
      inline fun setPrimitiveTopology(rhi::PrimitiveTopology val) { m_primitiveTopology = val; }


      fun desc(rhi::render_target &target) -> sptr<module::material_desc>;
  };

}
