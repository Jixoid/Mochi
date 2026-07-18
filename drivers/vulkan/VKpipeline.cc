/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/

#include "drivers/vulkan/VKdriver.hh"
#include "drivers/vulkan/VKconvert.hh"
#include "drivers/vulkan/VKpipeline.hh"
#include "drivers/vulkan/manager/VKdevice_manager.hh"
#include "drivers/vulkan/VKshader.hh"
#include <vulkan/vulkan_raii.hpp>
#include "mochi/debug/debug.hh"
#include "mochi/rhi/manager/device_manager.hh"
#include "mochi/rhi/manager/pipeline_manager.hh"
#include "mochi/rhi/pipeline.hh"
#include "mochi/rhi/shader.hh"
#include "vk_mem_alloc.h"



namespace mochi::rhi::vulkan
{

  extern "C" fun MochiRHI_MakePipelineMeta(PushConstantList push, VertexBindList vert) -> PipelineMeta* {
    return new VK_PipelineMeta(std::move(push), std::move(vert));
  }
  
  extern "C" fun MochiRHI_MakePipeline(
    rhi::mng::DeviceManager &device, rhi::mng::PipelineManager &pmng, u64 sign,
    PipelineMeta *meta, std::vector<sptr<Shader>> shaders,
    PolygonMode polymode, PrimitiveTopology primitiveTopology,
    Format color_format, Format depth_format
  ) -> Pipeline* {
    return new VK_Pipeline(device, pmng, sign, meta, std::move(shaders), polymode, primitiveTopology, color_format, depth_format);
  }


  constexpr inline fun align_size(u64 size, u64 alignment) { if (alignment == 0) return size; else return ((size+alignment -1) /alignment) *alignment; }


  VK_PipelineMeta::VK_PipelineMeta(PushConstantList push, VertexBindList vert) {
    u16 total_size{};
    rhi::ShaderStageFlags all_stages{};

    u64 max_align = 0;
    for (auto &p: push) {
      all_stages |= p.stage();
      for (auto &typ: p.types()) {
        max_align = std::max<u64>(max_align, typ.align());
        total_size = align_size(total_size, typ.align());
        total_size += typ.size() * typ.count();
      }
    }
    m_push_data_end = total_size; // offset after last byte, before struct-level padding
    total_size = align_size(total_size, max_align);

    if (total_size > 0) {
      vk_PushConstant.push_back(vk::PushConstantRange(
        VKConvert<ShaderStageFlags>(all_stages),
        0,
        total_size
      ));
    }


    u32 binding_idx{}, location_idx{};
    for (auto &ibuf: vert) {
      u32 v_offset{};

      for (auto &typ: ibuf.items()) {
        v_offset = align_size(v_offset, typ.align());
        
        for (u32 c{}; c < typ.count(); c++) {
          vk_vertexAttribute.push_back(vk::VertexInputAttributeDescription(
            location_idx++,
            binding_idx,
            VKConvert<Format>(typ.format()),
            v_offset
          ));
          v_offset += typ.size();
        }
      }

      vk_vertexBinding.push_back(vk::VertexInputBindingDescription(
        binding_idx++,
        ibuf.stride(),
        VKConvert<VertexInputRate>(ibuf.inputRate())
      ));

      //ibuf.index() = binding_idx;
    }

    
    m_push = std::move(push);
    m_vert = std::move(vert);
  }


  VK_Pipeline::VK_Pipeline(
    rhi::mng::DeviceManager &device, rhi::mng::PipelineManager &pmng, u64 sign,
    PipelineMeta *_meta, std::vector<sptr<Shader>> shaders,
    PolygonMode polymode, PrimitiveTopology primitiveTopology,
    Format __color_format, Format __depth_format
  )
    : vk_pipelineCache(nil), vk_layout(nil), vk_pipeline(nil)
  {
    auto meta = static_cast<VK_PipelineMeta*>(_meta);

    bool is_compute = (shaders.size() == 1 && shaders[0]->stage() == ShaderStage::Compute);


    // Pipeline Cache
    bool cache_hit = false;
    {
      auto cached = (sign != 0) ? pmng.loadCache(sign) : std::vector<u8>{};
      cache_hit = !cached.empty();

      vk::PipelineCacheCreateInfo cache_info(
        {},
        cached.size(),
        cached.empty() ? nullptr : cached.data()
      );

      if (cache_hit)
        ME_LOG_VERB("pipeline cache loaded")
      else
        ME_LOG_VERB("pipeline cache empty, building fresh")

      vk_pipelineCache = vk::raii::PipelineCache(static_cast<vulkan::mng::VK_DeviceManager&>(device).get(), cache_info);
    }



    vk::PipelineLayoutCreateInfo layout_info({}, {}, meta->pushConstant());
    vk_layout = vk::raii::PipelineLayout(static_cast<vulkan::mng::VK_DeviceManager&>(device).get(), layout_info);



    if (is_compute) {
      m_kind = PipelineKind::Compute;
      vk::PipelineShaderStageCreateInfo compute_stage(
        {}, VKConvert<ShaderStage>(shaders[0]->stage()), static_cast<vulkan::VK_Shader*>(shaders[0].get())->get(), shaders[0]->entry().c_str()
      );

      vk::ComputePipelineCreateInfo compute_info({}, compute_stage, *vk_layout);
      

      vk_pipeline = vk::raii::Pipeline(static_cast<vulkan::mng::VK_DeviceManager&>(device).get(), vk_pipelineCache, compute_info);
    } 
    else {
      m_kind = PipelineKind::Graphic;
      std::vector<vk::PipelineShaderStageCreateInfo> shader_stages;
      shader_stages.reserve(shaders.size());
      
      vk::DescriptorMappingSourcePushIndexEXT push_index = {};
      push_index.heapOffset = 0;
      push_index.pushOffset = meta->push_data_end() > sizeof(uint32_t) ? (meta->push_data_end() - sizeof(uint32_t)) : 0;
      push_index.heapIndexStride = static_cast<vulkan::mng::VK_DeviceManager&>(device).descriptor_size();
      push_index.heapArrayStride = 0;
      push_index.pEmbeddedSampler = nullptr;
      push_index.useCombinedImageSamplerIndex = VK_TRUE;
      push_index.samplerHeapOffset = 0;
      push_index.samplerPushOffset = push_index.pushOffset;
      push_index.samplerHeapIndexStride = static_cast<vulkan::mng::VK_DeviceManager&>(device).sampler_descriptor_size();
      push_index.samplerHeapArrayStride = 0;
      
      VkDescriptorSetAndBindingMappingEXT mapping = {};
      mapping.sType = VK_STRUCTURE_TYPE_DESCRIPTOR_SET_AND_BINDING_MAPPING_EXT;
      mapping.descriptorSet = 1;
      mapping.firstBinding = 0;
      mapping.bindingCount = 1;
      mapping.resourceMask = VK_SPIRV_RESOURCE_TYPE_COMBINED_SAMPLED_IMAGE_BIT_EXT;
      mapping.source = VK_DESCRIPTOR_MAPPING_SOURCE_HEAP_WITH_PUSH_INDEX_EXT;
      mapping.sourceData.pushIndex = push_index;
      
      VkShaderDescriptorSetAndBindingMappingInfoEXT mapping_info = {};
      mapping_info.sType = VK_STRUCTURE_TYPE_SHADER_DESCRIPTOR_SET_AND_BINDING_MAPPING_INFO_EXT;
      mapping_info.mappingCount = 1;
      mapping_info.pMappings = &mapping;

      for (auto &sh: shaders) {
        auto stage = vk::PipelineShaderStageCreateInfo(
          {}, VKConvert<ShaderStage>(sh->stage()), static_cast<vulkan::VK_Shader*>(sh.get())->get(), sh->entry().c_str()
        );
        if (sh->stage() == ShaderStage::Pixel) {
            stage.pNext = &mapping_info;
        }
        shader_stages.push_back(stage);
      }

      std::array<vk::DynamicState, 2> dynamic_states = { vk::DynamicState::eViewport, vk::DynamicState::eScissor };
      vk::PipelineDynamicStateCreateInfo dynamic_info({}, dynamic_states);

      vk::PipelineInputAssemblyStateCreateInfo input_assembly({}, VKConvert<PrimitiveTopology>(primitiveTopology), VK_FALSE);
      vk::PipelineViewportStateCreateInfo      viewport_state({}, 1, nil, 1, nil);
      vk::PipelineMultisampleStateCreateInfo   multisampling({}, vk::SampleCountFlagBits::e1, VK_FALSE);

      vk::PipelineRasterizationStateCreateInfo rasterizer(
        {}, VK_FALSE, VK_FALSE, VKConvert<PolygonMode>(polymode), 
        vk::CullModeFlagBits::eBack, vk::FrontFace::eCounterClockwise,
        VK_FALSE, 0.0f, 0.0f, 0.0f, 1.0f
      );

      vk::PipelineColorBlendAttachmentState blend_attach(
        VK_FALSE, vk::BlendFactor::eOne, vk::BlendFactor::eZero, vk::BlendOp::eAdd,
        vk::BlendFactor::eOne, vk::BlendFactor::eZero, vk::BlendOp::eAdd,
        vk::ColorComponentFlagBits::eR | vk::ColorComponentFlagBits::eG | vk::ColorComponentFlagBits::eB | vk::ColorComponentFlagBits::eA
      );
      vk::PipelineColorBlendStateCreateInfo color_blending({}, VK_FALSE, vk::LogicOp::eCopy, 1, &blend_attach);

      vk::PipelineDepthStencilStateCreateInfo depth_stencil(
        {}, VK_TRUE, VK_TRUE, vk::CompareOp::eLess,
        VK_FALSE, VK_FALSE, {}, {}, 0.0f, 1.0f
      );

      auto color_format = VKConvert<Format>(__color_format);
      auto depth_format = VKConvert<Format>(__depth_format);
      vk::PipelineRenderingCreateInfo rendering_info(
        0, 1, &color_format,
        depth_format, vk::Format::eUndefined
      );

      VkPipelineCreateFlags2CreateInfo create_flags2 = {};
      create_flags2.sType = VK_STRUCTURE_TYPE_PIPELINE_CREATE_FLAGS_2_CREATE_INFO;
      create_flags2.flags = VK_PIPELINE_CREATE_2_DESCRIPTOR_HEAP_BIT_EXT;
      create_flags2.pNext = &rendering_info;

      auto VertexInput = meta->vertexBinding();
      vk::GraphicsPipelineCreateInfo pipeline_info(
        {}, shader_stages, &VertexInput, &input_assembly, nil,
        &viewport_state, &rasterizer, &multisampling, &depth_stencil,
        &color_blending, &dynamic_info, nullptr // VK_NULL_HANDLE for Descriptor Heaps
      );
      pipeline_info.pNext = &create_flags2;


      vk_pipeline = vk::raii::Pipeline(static_cast<vulkan::mng::VK_DeviceManager&>(device).get(), vk_pipelineCache, pipeline_info);
    }


    // Save cache
    if (sign != 0 && !cache_hit) {
      auto cache_data = vk_pipelineCache.getData();
      pmng.saveCache(sign, std::span<const u8>(cache_data.data(), cache_data.size()));
    }
  }
  
}
