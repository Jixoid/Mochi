/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#include "mochi/basis.hh"
#include "mochi/except.hh"
#include "mochi/rhi/rhi.hh"
#include "mochi/rhi/vtype.hh"
#include "mochi/types.hh"
#include "mochi/asset/mesh.hh"
#include "mochi/asset/texture.hh"
#include "mochi/ecs/camera.hh"
#include "mochi/ecs/point_light.hh"
#include "mochi/rhi/convert.hh"
#include "mochi/rhi/pipeline.hh"
#include "mochi/rhi/render_target.hh"
#include "mochi/rhi/descset.hh"
#include "mochi/rhi/slotPush.hh"
#include "mochi/rhi/slotDesc.hh"
#include "mochi/rhi/slotVertex.hh"
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


    auto Transform = rhi::info<rhi::slotPush>::make(rhi::vt::make<mat4<f32>>(), flags(rhi::ShaderStage::Vertex) | rhi::ShaderStage::Pixel);
    
    auto Vertex = rhi::info<rhi::slotVertex>::make(asset::vertex_i, rhi::VertexInputRate::PerVertex);

    auto Camera = rhi::info<rhi::slotDesc>::make(ecs::camera_i, rhi::DescriptorType::UniformBuffer, flags(rhi::ShaderStage::Vertex) | rhi::ShaderStage::Pixel);
    auto Light = rhi::info<rhi::slotDesc>::make(ecs::point_light_i, rhi::DescriptorType::UniformBuffer, flags(rhi::ShaderStage::Vertex) | rhi::ShaderStage::Pixel);
    auto Texture = rhi::info<rhi::slotDesc>::make(nil, rhi::DescriptorType::TextureSampler, rhi::ShaderStage::Pixel);


    auto Set0 = rhi::info<rhi::descset>::make({Camera, Light});
    auto Set1 = rhi::info<rhi::descset>::make({Texture});


    shader_color_i = rhi::info<rhi::pipeline>::make({Transform}, {Vertex}, {Set0});
    shader_texture_i = rhi::info<rhi::pipeline>::make({Transform}, {Vertex}, {Set0, Set1});
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
    static std::unordered_map<material_method, std::string> ShSource = {
      {material_method::mmBare, "embed://bare"},
      {material_method::mmPBR,  "embed://pbr"},
    };

    std::unordered_map<material_albedo, sptr<rhi::info<rhi::pipeline>>> InAlbedo = {
      {material_albedo::maColor,   shader_color_i},
      {material_albedo::maTexture, shader_texture_i},
    };

    static std::unordered_map<material_albedo, std::string> ShAlbedo = {
      {material_albedo::maColor,   "WITH_COLOR"},
      {material_albedo::maTexture, "WITH_TEXTURE"},
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



    auto vert = compile_shader(rhi::ShaderStage::Vertex, vfs::resolve_ro(ShSource[props.method]+".vert").get(), {ShAlbedo[props.albedo]});
    auto frag = compile_shader(rhi::ShaderStage::Pixel,  vfs::resolve_ro(ShSource[props.method]+".frag").get(), {ShAlbedo[props.albedo]});

    if (!vert.has_value() || !frag.has_value())
      throw mochi::rhi_error("Shaders failed to compile.");

    
    auto pipe = rhi::pipeline::make(
      m_device, InAlbedo[props.albedo], {*vert, *frag},
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

    
    auto ret = make_sptr(new material_desc{pipe, std::move(desc_sets)});
    m_materials[props] = ret;
    return ret;
  }

}
