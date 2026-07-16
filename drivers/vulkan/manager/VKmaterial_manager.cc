/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#include "drivers/vulkan/manager/VKmaterial_manager.hh"
#include "mochi/basis.hh"
#include "mochi/except.hh"
#include "mochi/rhi/render_target.hh"
#include "mochi/types.hh"
#include "mochi/rhi/manager/device_manager.hh"
#include "mochi/rhi/manager/material_manager.hh"
#include "mochi/vfs/vfs.hh"
#include <vulkan/vulkan_raii.hpp>



namespace mochi::rhi::vulkan
{

  extern "C" fun MochiRHI_MakeMaterialManager(rhi::DeviceManager &device, rhi::ShaderManager &smng) -> rhi::MaterialManager* {
    return new VK_MaterialManager(device, smng);
  }


  VK_MaterialManager::VK_MaterialManager(rhi::DeviceManager &device, ShaderManager &smng)
    : rhi::MaterialManager(device, smng)
  {}

  
  fun VK_MaterialManager::getMaterialDesc(rhi::render_target &target, MaterialProps props) -> sptr<MaterialDesc> {
    // Info Pipeline
    rhi::PushConstantList ipush;

    
    // push: model
    ipush.push_back(rhi::PushConstantSlot(rhi::ShaderStage::Vertex, rhi::vt::make_list<mat4x4<f32>>()));

    // push: vertex
    ipush.push_back(rhi::PushConstantSlot(rhi::ShaderStage::Vertex, rhi::vt::make_list<u64>()));
    
    // push: instance
    if (props.count == MaterialCount::Multi)
      ipush.push_back(rhi::PushConstantSlot(rhi::ShaderStage::Vertex, rhi::vt::make_list<u64>()));



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

    if (it != m_materials.end())
      return it->second;


    std::vector<std::string> Macros = {
      ShAlbedo[props.albedo],
      ShCount[props.count],
    };

    auto sign = std::hash<MaterialProps>()(props);

    auto vert = m_smng.compileGLSL(sign, rhi::ShaderStage::Vertex, vfs::open_map(ShSource[props.method]+".vert")->span(), Macros);
    auto frag = m_smng.compileGLSL(sign, rhi::ShaderStage::Pixel,  vfs::open_map(ShSource[props.method]+".frag")->span(), Macros);

    if (!vert || !frag)
      throw mochi::rhi_error("Shaders failed to compile.");


    auto ipipe = rhi::PipelineMeta::make(ipush, {},{});
    
    auto pipe = rhi::Pipeline::make(
      m_device, ipipe.get(), {vert, frag},
      props.polymode, props.primitiveTopology,
      static_cast<rhi::Format>(static_cast<uint32_t>(target.color_format)),
      static_cast<rhi::Format>(target.depth_format)
    );

    
    auto ret = make_sptr(new MaterialDesc{ipipe, pipe});
    m_materials[props] = ret;

    return ret;
  };
  
}
