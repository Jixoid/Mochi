/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/

#pragma once

#include "mochi/basis.hh"
#include "mochi/rhi/utility/pipeline_cache_utility.hh"
#include "mochi/rhi/utility/shader_cache_utility.hh"
#include "mochi/rhi/utility/shader_compile_utility.hh"
#include "mochi/rhi/manager/device.hh"
#include "mochi/rhi/render_target.hh"
#include "mochi/types.hh"
#include "mochi/rhi/pipeline.hh"
#include <functional>
#include <vulkan/vulkan_raii.hpp>



namespace mochi::utility
{
  // Enums
  enum struct MaterialMethod: u8 { Bare, PBR };
  enum struct MaterialAlbedo: u8 { Color, Texture };
  enum struct MaterialCount:  u8 { Single, Multi };
  

  // Helpers
  struct MaterialProps {
    MaterialMethod method;
    MaterialAlbedo albedo;
    MaterialCount  count;

    rhi::PolygonMode polymode;
    rhi::PrimitiveTopology primitiveTopology;

    rhi::Format color_format, depth_format;

    fun operator ==(const MaterialProps &it) const -> bool = default;
  };
  
  struct MaterialDesc {
    sptr<rhi::PipelineMeta> pipemeta;
    sptr<rhi::Pipeline> pipeline;
  };

}

template<>
struct std::hash<mochi::utility::MaterialProps> {
  fun operator()(const mochi::utility::MaterialProps &d) const noexcept -> std::size_t {
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
    hash_combine(d.count);
    
    hash_combine(d.polymode);
    hash_combine(d.primitiveTopology);

    hash_combine(d.color_format);
    hash_combine(d.depth_format);

    return seed;
  }
};

namespace mochi::utility
{
  // Interface
  struct MaterialUtility: noncopy {
    public:
      MaterialUtility(rhi::Device &device, rhi::ShaderCompileUtility &smng, rhi::ShaderCacheUtility &hmng, rhi::PipelineCacheUtility &pmng)
        : m_dmng(device)
        , m_smng(smng)
        , m_hmng(hmng)
        , m_pmng(pmng)
      {}
      
    private:
      rhi::Device &m_dmng;
      rhi::ShaderCompileUtility &m_smng;
      rhi::ShaderCacheUtility   &m_hmng;
      rhi::PipelineCacheUtility &m_pmng;
      
      std::unordered_map<MaterialProps, sptr<MaterialDesc>> m_materials;
    
    public:
      fun getMaterialDesc(rhi::RenderTarget &target, MaterialProps props) -> sptr<MaterialDesc>;
  };
  
}
