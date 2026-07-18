/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#include "mochi/basis.hh"
#include "mochi/debug/debug.hh"
#include "mochi/except.hh"
#include "mochi/manager/material_manager.hh"
#include "mochi/rhi/render_target.hh"
#include "mochi/types.hh"
#include "mochi/vfs/vfs.hh"
#include <format>
#include <vulkan/vulkan_raii.hpp>



namespace mochi::mng
{
  
  fun MaterialManager::getMaterialDesc(rhi::RenderTarget &target, MaterialProps props) -> sptr<MaterialDesc> {
    // Info Pipeline
    rhi::PushConstantList ipush;

    
    // push: model
    ipush.push_back(rhi::PushConstantSlot(rhi::ShaderStage::Vertex | rhi::ShaderStage::Pixel, rhi::vt::make_list<mat4x4<f32>>()));

    // push: vertex
    ipush.push_back(rhi::PushConstantSlot(rhi::ShaderStage::Vertex | rhi::ShaderStage::Pixel, rhi::vt::make_list<u64>()));
    
    // push: camera
    ipush.push_back(rhi::PushConstantSlot(rhi::ShaderStage::Vertex | rhi::ShaderStage::Pixel, rhi::vt::make_list<u64>()));
    
    // push: light
    ipush.push_back(rhi::PushConstantSlot(rhi::ShaderStage::Vertex | rhi::ShaderStage::Pixel, rhi::vt::make_list<u64>()));

    // push: instance
    if (props.count == MaterialCount::Multi)
      ipush.push_back(rhi::PushConstantSlot(rhi::ShaderStage::Vertex | rhi::ShaderStage::Pixel, rhi::vt::make_list<u64>()));

    // push: texture_id
    ipush.push_back(rhi::PushConstantSlot(rhi::ShaderStage::Vertex | rhi::ShaderStage::Pixel, rhi::vt::make_list<u32>()));
    

    // Pipeline
    static std::unordered_map<MaterialMethod, std::string> ShSource = {
      {MaterialMethod::Bare, "embed://bare"},
      {MaterialMethod::PBR,  "embed://pbr"},
    };

    static std::unordered_map<MaterialAlbedo, std::string> ShAlbedo = {
      {MaterialAlbedo::Color,   "WITH_COLOR"},
      {MaterialAlbedo::Texture, "WITH_TEXTURE"},
    };

    static std::unordered_map<MaterialCount, std::string> ShCount = {
      {MaterialCount::Single, "WITH_SINGLE_INST"},
      {MaterialCount::Multi,  "WITH_MULTI_INST"},
    };


    auto it = m_materials.find(props);

    if (it != m_materials.end()) {
      return it->second;
    }

    ME_LOG_VERB("material building (method={}, albedo={})", static_cast<u32>(props.method), static_cast<u32>(props.albedo));

    std::vector<std::string> Macros = {
      ShAlbedo[props.albedo],
      ShCount[props.count],
    };


    auto hash_combine = [](usize seed, auto &&v) -> usize {
      using T = std::decay_t<decltype(v)>;
      std::size_t h;
      
      if constexpr (std::is_enum_v<T>)
        h = std::hash<std::underlying_type_t<T>>{}(std::to_underlying(v));
      else
        h = std::hash<T>{}(v);
      
      return seed ^ (h + 0x9e3779b97f4a7c15 + (seed << 6) + (seed >> 2));
    };

    auto sign = std::hash<MaterialProps>()(props);
    auto sign_v = hash_combine(sign, rhi::ShaderStage::Vertex);
    auto sign_p = hash_combine(sign, rhi::ShaderStage::Pixel);

    auto vert = m_smng.compileGLSL(sign_v, rhi::ShaderStage::Vertex, vfs::open_map(ShSource[props.method]+".vert")->span(), Macros);
    auto frag = m_smng.compileGLSL(sign_p, rhi::ShaderStage::Pixel,  vfs::open_map(ShSource[props.method]+".frag")->span(), Macros);

    if (!vert || !frag)
      throw mochi::rhi_error("Shaders failed to compile.");


    auto ipipe = rhi::PipelineMeta::make(ipush, {});
    
    auto pipe = rhi::Pipeline::make(
      m_dmng, m_pmng, sign, ipipe.get(), {vert, frag},
      props.polymode, props.primitiveTopology,
      target.color_format(),
      target.depth_format()
    );

    
    auto ret = make_sptr(new MaterialDesc{ipipe, pipe});
    m_materials[props] = ret;

    return ret;
  };
  
}
