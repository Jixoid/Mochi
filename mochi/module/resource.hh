/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#pragma once

#include "mochi/basis.hh"
#include "mochi/rhi/render_target.hh"
#include "mochi/rhi/rhi.hh"
#include "mochi/types.hh"
#include "mochi/module/device.hh"
#include "mochi/rhi/pipeline.hh"
#include "vulkan/vulkan.hpp"
#include <expected>
#include <functional>
#include <string>
#include <unordered_map>
#include <vulkan/vulkan_raii.hpp>
#include <vector>



namespace mochi::module
{

  enum material_method: u8 { mmBare, mmPBR };
  enum material_albedo: u8 { maColor, maTexture };
  
  struct material_props {
    material_method method;
    material_albedo albedo;
    asset::texture2 *texture{};
    rhi::PolygonMode polymode;
    rhi::PrimitiveTopology primitiveTopology;

    vk::Format __color_format, __depth_format;

    inline fun operator ==(const material_props &it) const -> bool {
      return (method == it.method)
        && (albedo == it.albedo) && (texture == it.texture)
        && (polymode == it.polymode) && (primitiveTopology == it.primitiveTopology)
        && (__color_format == it.__color_format) && (__depth_format == it.__depth_format);
    }
  };
  
  struct material_desc {
    sptr<rhi::pipeline> pipeline;
    std::vector<vk::raii::DescriptorSet> desc_sets;
  };

}


template<>
struct std::hash<mochi::module::material_props> {
  std::size_t operator()(const mochi::module::material_props &d) const noexcept {
    std::size_t seed = 0;

    auto hash_combine = [&seed](auto &&v) {
      using T = std::decay_t<decltype(v)>;
      std::size_t h;
      
      if constexpr (std::is_enum_v<T>)
        h = std::hash<std::underlying_type_t<T>>{}(std::to_underlying(v));
      else
        h = std::hash<T>{}(v);
      
      seed ^= h + 0x9e3779b97f4a7c15 + (seed << 6) + (seed >> 2);
    };

    hash_combine(d.method);
    hash_combine(d.albedo);
    hash_combine(d.texture);
    
    hash_combine(d.polymode);
    hash_combine(d.primitiveTopology);

    hash_combine(d.__color_format);
    hash_combine(d.__depth_format);

    return seed;
  }
};


namespace mochi::module
{

  /** @brief Manages resource allocations such as descriptor pools and sets. */
  struct resource
  {
    public:
      /**
       * @brief Initialize a new resource module.
       * @param device The logical device.
       */
      explicit resource(module::device &device, module::memory &memory);
      

    private:
      module::device &m_device;
      module::memory &m_memory;
      std::vector<vk::raii::DescriptorPool> m_pools;

      std::unordered_map<material_props, sptr<material_desc>> m_materials;

      fun create_pool() -> vk::raii::DescriptorPool&;
      fun allocate_descriptor_set(vk::DescriptorSetLayout layout) -> vk::raii::DescriptorSet;


    private:
      rhi::info<rhi::pipeline> shader_color_i, shader_texture_i;

    public:
      fun compile_shader(rhi::ShaderStage stage, std::istream *file, std::vector<std::string> macros) -> std::expected<sptr<rhi::shader>, std::string>;

      fun get_or_new_material_desc(rhi::render_target &target, material_props props) -> sptr<material_desc>;
  };
  
}
