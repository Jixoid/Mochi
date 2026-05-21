/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#include "mochi/basis.hh"
#include "mochi/except.hh"
#include "mochi/types.hh"
#include "mochi/rhi/rhi.hh"
#include "mochi/rhi/vtype.hh"
#include "mochi/asset/mesh.hh"
#include "mochi/asset/texture.hh"
#include "mochi/ecs/camera.hh"
#include "mochi/ecs/point_light.hh"
#include "mochi/rhi/convert.hh"
#include "mochi/rhi/pipeline.hh"
#include "mochi/rhi/render_target.hh"
#include "mochi/rhi/listPush.hh"
#include "mochi/rhi/listDesc.hh"
#include "mochi/rhi/slotPush.hh"
#include "mochi/rhi/slotDesc.hh"
#include "mochi/module/resource.hh"
#include "mochi/vfs/vfs.hh"
#include "vulkan/vulkan.hpp"
#include <expected>
#include <shaderc/shaderc.h>
#include <shaderc/shaderc.hpp>
#include <span>
#include <spanstream>
#include <string>
#include <unordered_map>



namespace mochi::module
{

  resource::resource(module::device &device, module::memory &memory)
    : m_device(device)
    , m_memory(memory)
  {
    create_pool();
  }
  


  fun resource::create_pool() -> vk::raii::DescriptorPool&
  {
    std::array<vk::DescriptorPoolSize, 2> pool_sizes = {
      vk::DescriptorPoolSize(vk::DescriptorType::eUniformBuffer, 1000),
      vk::DescriptorPoolSize(vk::DescriptorType::eCombinedImageSampler, 1000)
    };

    vk::DescriptorPoolCreateInfo pool_info(
      vk::DescriptorPoolCreateFlagBits::eFreeDescriptorSet, 
      1000, 
      pool_sizes
    );

    m_pools.push_back(vk::raii::DescriptorPool(m_device.vdevice(), pool_info));
    return m_pools.back();
  }



  fun resource::allocate_descriptor_set(vk::DescriptorSetLayout layout) -> vk::raii::DescriptorSet
  {
    vk::DescriptorSetAllocateInfo alloc_info(*m_pools.back(), 1, &layout);
    
    try {
      vk::raii::DescriptorSets sets(m_device.vdevice(), alloc_info);
      return std::move(sets.front());
    }
    catch(const vk::OutOfPoolMemoryError &e) {
      auto &new_pool = create_pool();
      vk::DescriptorSetAllocateInfo new_alloc_info(*new_pool, 1, &layout);
      vk::raii::DescriptorSets sets(m_device.vdevice(), new_alloc_info);
      return std::move(sets.front());
    }
    catch(const vk::FragmentedPoolError &e) {
      auto &new_pool = create_pool();
      vk::DescriptorSetAllocateInfo new_alloc_info(*new_pool, 1, &layout);
      vk::raii::DescriptorSets sets(m_device.vdevice(), new_alloc_info);
      return std::move(sets.front());
    }
  }


  fun resource::compile_shader(rhi::ShaderStage stage, std::istream *file, std::vector<std::string> macros) -> std::expected<sptr<rhi::shader>, std::string>
  {
    static std::unordered_map<rhi::ShaderStage, shaderc_shader_kind> ToKind = {
      {rhi::ShaderStage::Vertex,  shaderc_glsl_vertex_shader},
      {rhi::ShaderStage::Pixel,   shaderc_glsl_fragment_shader},
      {rhi::ShaderStage::Compute, shaderc_glsl_compute_shader},
    };


    shaderc::Compiler compiler;
    shaderc::CompileOptions options;

    options.SetTargetEnvironment(shaderc_target_env_vulkan, shaderc_env_version_vulkan_1_4);
    options.SetOptimizationLevel(shaderc_optimization_level_performance);

    for (auto &M: macros) options.AddMacroDefinition(M);


    shaderc::SpvCompilationResult module;

    if (auto span = dynamic_cast<std::ispanstream*>(file)) {
      module = compiler.CompileGlslToSpv(span->span().data(), span->span().size(), ToKind[stage], "main.glsl", options);
    }
    else {
      std::string res;
      std::getline(*file, res, '\0');

      module = compiler.CompileGlslToSpv(res.data(), res.size(), ToKind[stage], "main.glsl", options);
    }


    if (module.GetCompilationStatus() != shaderc_compilation_status_success)
      return std::unexpected("Shader compilation error: " + module.GetErrorMessage());

    
    return rhi::shader::make(m_device, stage, std::span<u32>((u32*)module.begin(), module.end()), "main");
  }


  fun resource::get_or_new_material_desc(rhi::render_target &target, material_props props) -> sptr<material_desc>
  {
    // Info Pipeline
    std::vector<rhi::info<rhi::slotPush>> ipush;
    std::vector<rhi::info<rhi::slotDesc>> idescs_0;
    std::vector<rhi::info<rhi::slotDesc>> idescs_1;



    // push: model
    ipush.push_back(rhi::vt::make<mat4<f32>>());

    // push: vertex
    ipush.push_back(rhi::vt::make<u64>());
    
    // push: instance
    if (props.count == material_count::mcMulti)
      ipush.push_back(rhi::vt::make<u64>());



    // desc: set 0
    idescs_0.push_back(rhi::info<rhi::slotDesc>(ecs::camera_i, rhi::DescriptorType::UniformBuffer, flags(rhi::ShaderStage::Vertex) | rhi::ShaderStage::Pixel));
    idescs_0.push_back(rhi::info<rhi::slotDesc>(ecs::point_light_i, rhi::DescriptorType::UniformBuffer, flags(rhi::ShaderStage::Vertex) | rhi::ShaderStage::Pixel));
    
    
    // desc: set 1
    idescs_1.push_back(rhi::info<rhi::slotDesc>(rhi::info<rhi::buffer>(1, {}), rhi::DescriptorType::TextureSampler, rhi::ShaderStage::Pixel));



    // prepare
    auto Push = rhi::info<rhi::listPush>(rhi::ShaderStage::Vertex, std::move(ipush));
    auto Set0 = rhi::info<rhi::listDesc>(std::move(idescs_0));
    auto Set1 = rhi::info<rhi::listDesc>(std::move(idescs_1));


    // descset
    std::vector<rhi::info<rhi::listDesc>> iDescSet;
    
    if (!Set0.idescs().empty()) iDescSet.push_back(Set0);
    if (!Set1.idescs().empty()) iDescSet.push_back(Set1);




    // Pipeline
    static std::unordered_map<material_method, std::string> ShSource = {
      {material_method::mmBare, "embed://bare"},
      {material_method::mmPBR,  "embed://pbr"},
    };

    static std::unordered_map<material_albedo, std::string> ShAlbedo = {
      {material_albedo::maColor,   "WITH_COLOR"},
      {material_albedo::maTexture, "WITH_TEXTURE"},
    };

    static std::unordered_map<material_count, std::string> ShCount = {
      {material_count::mcSingle, "WITH_SINGLE_INST"},
      {material_count::mcMulti,  "WITH_MULTI_INST"},
    };

    auto bind_uniform = [this](u32 binding, vk::raii::DescriptorSet &desc_set, rhi::buffer *buf){
      vk::DescriptorBufferInfo buffer_info(buf->get(), 0, buf->size());

      vk::WriteDescriptorSet descriptor_write(
        *desc_set,
        binding,
        0,
        1,
        vk::DescriptorType::eUniformBuffer,
        nil,
        &buffer_info,
        nil
      );

      m_device.vdevice().updateDescriptorSets(descriptor_write, nil);
    };

    auto bind_texture = [this](u32 binding, vk::raii::DescriptorSet &desc_set, vk::ImageView image_view, vk::Sampler sampler) -> void
    {
      vk::DescriptorImageInfo image_info(
        sampler,
        image_view,
        vk::ImageLayout::eShaderReadOnlyOptimal
      );

      vk::WriteDescriptorSet descriptor_write(
        *desc_set,
        binding,
        0,
        1,
        vk::DescriptorType::eCombinedImageSampler,
        &image_info,
        nil,
        nil
      );

      m_device.vdevice().updateDescriptorSets(descriptor_write, nullptr);
    };



    props.__color_format = target.color_format;
    props.__depth_format = target.depth_format;

    auto it = m_materials.find(props);

    if (it != m_materials.end())
      return it->second;



    std::vector<std::string> Macros = {
      ShAlbedo[props.albedo],
      ShCount[props.count],
    };

    auto vert = compile_shader(rhi::ShaderStage::Vertex, vfs::resolve_ro(ShSource[props.method]+".vert").get(), Macros);
    auto frag = compile_shader(rhi::ShaderStage::Pixel,  vfs::resolve_ro(ShSource[props.method]+".frag").get(), Macros);

    if (!vert.has_value() || !frag.has_value())
      throw mochi::rhi_error("Shaders failed to compile.");


    auto ipipe = rhi::info<rhi::pipeline>(
      {Push},
      {},
      iDescSet
    );
    
    auto pipe = rhi::pipeline::make(
      m_device, ipipe, {*vert, *frag},
      props.polymode, props.primitiveTopology,
      static_cast<rhi::Format>(target.color_format), static_cast<rhi::Format>(target.depth_format)
    );

    std::vector<vk::raii::DescriptorSet> desc_sets;
    desc_sets.reserve(pipe->desc_layouts().size());

    for (auto &descset: pipe->desc_layouts())
      desc_sets.push_back(allocate_descriptor_set(*descset));


    bind_uniform(0, desc_sets[0], m_memory.camera_ubo().get());
    bind_uniform(1, desc_sets[0], m_memory.light_ubo().get());

    if (props.albedo == material_albedo::maTexture)
      bind_texture(0, desc_sets[1], props.texture->data()->view(), props.texture->data()->sampler());

    
    auto ret = make_sptr(new material_desc{ipipe, pipe, std::move(desc_sets)});
    m_materials[props] = ret;
    return ret;
  }

}
