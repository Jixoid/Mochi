/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#include "mochi/basis.hh"
#include "mochi/rhi/rhi.hh"
#include "mochi/types.hh"
#include "mochi/rhi/convert.hh"
#include "mochi/rhi/pipeline.hh"
#include "mochi/rhi/listDesc.hh"
#include "mochi/rhi/listPush.hh"
#include "mochi/rhi/slotPush.hh"
#include "mochi/rhi/slotDesc.hh"
#include "mochi/rhi/slotVertex.hh"
#include "mochi/module/device.hh"
#include <vector>
#include <vulkan/vulkan.hpp>
#include <vulkan/vulkan_raii.hpp>



namespace mochi::rhi
{

  constexpr inline fun align_size(u64 size, u64 alignment) { if (alignment == 0) return size; else return ((size + alignment - 1) / alignment) * alignment; }

  info<pipeline>::info(std::vector<info<listPush>> pushlist, std::vector<info<slotVertex>> vertex, std::vector<info<rhi::listDesc>> descset)
  {
    u16 off{};
    for (auto &push: pushlist) {
      u16 size{};

      auto legoff = off;
      for (auto &typ: push.ipush()) {
        off = align_size(off, typ.type().align());
        off += typ.type().size() * typ.type().count();
      }

      vk_PushConstant.push_back(vk::PushConstantRange(
        VKConvert<ShaderStageFlags>(push.stage()),
        legoff,
        off-legoff
      ));
    }



    u32 binding_idx{}, location_idx{};
    for (auto &ibuf: vertex) {
      u32 v_offset{};

      for (auto &typ: ibuf.ibuf()->items()) {
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
        ibuf.ibuf()->stride(),
        VKConvert<VertexInputRate>(ibuf.inputRate())
      ));

      ibuf.index() = binding_idx;
    }
  
    

    vk_DescBindings.reserve(descset.size());

    for (auto &desc: descset) 
    {
      std::vector<vk::DescriptorSetLayoutBinding> current_set_bindings;
      
      u32 binding_idx{};
      for (auto &slot: desc.idescs()) 
      {
        current_set_bindings.push_back(vk::DescriptorSetLayoutBinding(
          binding_idx++,
          VKConvert<DescriptorType>(slot.kind()),
          1,
          VKConvert<ShaderStageFlags>(slot.stage()),
          nil
        ));
      }
      
      vk_DescBindings.push_back(std::move(current_set_bindings));
    }

    
    m_push    = std::move(pushlist);
    m_vertex  = std::move(vertex);
    m_descset = std::move(descset);
  }


  pipeline::pipeline(
    module::device &device, rhi::info<pipeline> info, std::vector<sptr<shader>> shaders,
    PolygonMode polymode, PrimitiveTopology primitiveTopology,
    Format __color_format, Format __depth_format
  )
    : vk_layout(nil), vk_pipeline(nil)
    , m_dbg_info(info)
  {
    bool is_compute = (shaders.size() == 1 && shaders[0]->stage() == ShaderStage::Compute);


    std::vector<vk::DescriptorSetLayout> raw_layouts;


    for (auto &bindings_for_set: info.descBindings()) {
      vk::DescriptorSetLayoutCreateInfo set_info({}, bindings_for_set);
      
      vk_desc_layouts.push_back(vk::raii::DescriptorSetLayout(device.vdevice(), set_info));
      
      raw_layouts.push_back(*vk_desc_layouts.back());
    }
    

    vk::PipelineLayoutCreateInfo layout_info({}, raw_layouts, info.pushConstant());
    vk_layout = vk::raii::PipelineLayout(device.vdevice(), layout_info);



    if (is_compute) 
    {
      m_kind = PipelineKind::Compute;
      vk::PipelineShaderStageCreateInfo compute_stage(
        {}, VKConvert<ShaderStage>(shaders[0]->stage()), shaders[0]->module(), shaders[0]->entry().c_str()
      );

      vk::ComputePipelineCreateInfo compute_info({}, compute_stage, *vk_layout);
      

      vk_pipeline = vk::raii::Pipeline(device.vdevice(), nil, compute_info);
    } 
    else 
    {
      m_kind = PipelineKind::Graphic;
      std::vector<vk::PipelineShaderStageCreateInfo> shader_stages;
      shader_stages.reserve(shaders.size());
      for (auto &sh: shaders) {
        shader_stages.push_back(vk::PipelineShaderStageCreateInfo(
          {}, VKConvert<ShaderStage>(sh->stage()), sh->module(), sh->entry().c_str()
        ));
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

      auto VertexInput = info.vertexBinding();
      vk::GraphicsPipelineCreateInfo pipeline_info(
        {}, shader_stages, &VertexInput, &input_assembly, nil,
        &viewport_state, &rasterizer, &multisampling, &depth_stencil,
        &color_blending, &dynamic_info, *vk_layout
      );
      pipeline_info.pNext = &rendering_info;


      vk_pipeline = vk::raii::Pipeline(device.vdevice(), nil, pipeline_info);
    }
  }
  
}
