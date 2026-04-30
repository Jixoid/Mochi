/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#include "qvk/pipeline.hh"
#include "qvk/engine.hh"
#include "qvk/geometry.hh"
#include "qvk/meta.hh"
#include "qvk/shader.hh"
#include "qvk/swapchain.hh"
#include <vulkan/vulkan_raii.hpp>



namespace qvk
{

  pipeline::pipeline(qvk::engine &engine, qvk::shader vert, qvk::shader frag)
    : vk_layout(Nil), vk_pipeline(Nil)
  {
    // Push Constant
    auto PushConstant = []()
    {
      return std::vector<vk::PushConstantRange>
      {
        vk::PushConstantRange(vk::ShaderStageFlagBits::eVertex, 0, sizeof(qvk::mat4<f32>)),
      };
    }();






    // Add Shaders
    std::array<vk::PipelineShaderStageCreateInfo, 2> shader_stages = {
      vk::PipelineShaderStageCreateInfo({}, vk::ShaderStageFlagBits::eVertex, *vert.module(), "main"),
      vk::PipelineShaderStageCreateInfo({}, vk::ShaderStageFlagBits::eFragment, *frag.module(), "main")
    };



    // Dynamic States
    std::array<vk::DynamicState, 2> dynamic_states = { vk::DynamicState::eViewport, vk::DynamicState::eScissor };
    vk::PipelineDynamicStateCreateInfo dynamic_info({}, dynamic_states);



    // Fixed-Function States
    auto binding_desc = qvk::vertex::get_binding_description();
    auto attr_descs   = qvk::vertex::get_attribute_descriptions();

    vk::PipelineVertexInputStateCreateInfo vertex_input({}, binding_desc, attr_descs);

    vk::PipelineInputAssemblyStateCreateInfo input_assembly({}, vk::PrimitiveTopology::eTriangleList, VK_FALSE);
    vk::PipelineViewportStateCreateInfo      viewport_state({}, 1, nullptr, 1, nullptr);
    vk::PipelineMultisampleStateCreateInfo   multisampling({}, vk::SampleCountFlagBits::e1, VK_FALSE);



    // Create Rasterizer
    vk::PipelineRasterizationStateCreateInfo rasterizer(
      {}, VK_FALSE, VK_FALSE, vk::PolygonMode::eFill, 
      vk::CullModeFlagBits::eBack, vk::FrontFace::eClockwise, 
      VK_FALSE, 0.0f, 0.0f, 0.0f, 1.0f
    );

    vk::PipelineColorBlendAttachmentState blend_attach(
      VK_FALSE, vk::BlendFactor::eOne, vk::BlendFactor::eZero, vk::BlendOp::eAdd,
      vk::BlendFactor::eOne, vk::BlendFactor::eZero, vk::BlendOp::eAdd,
      vk::ColorComponentFlagBits::eR | vk::ColorComponentFlagBits::eG | vk::ColorComponentFlagBits::eB | vk::ColorComponentFlagBits::eA
    );
    vk::PipelineColorBlendStateCreateInfo color_blending({}, VK_FALSE, vk::LogicOp::eCopy, 1, &blend_attach);



    // Create Layout
    vk::PipelineLayoutCreateInfo layout_info({}, {}, PushConstant);
    vk_layout = vk::raii::PipelineLayout(engine.sub<device>().vdevice(), layout_info);



    // Dynamic Rendering
    vk::PipelineRenderingCreateInfo rendering_info(0, 1, &engine.sub<swapchain>().format(), vk::Format::eUndefined, vk::Format::eUndefined);


    // Create Pipeline
    vk::GraphicsPipelineCreateInfo pipeline_info(
      {}, shader_stages, &vertex_input, &input_assembly, nullptr,
      &viewport_state, &rasterizer, &multisampling, nullptr,
      &color_blending, &dynamic_info, *vk_layout
    );
    pipeline_info.pNext = &rendering_info;

    vk_pipeline = vk::raii::Pipeline(engine.sub<device>().vdevice(), nullptr, pipeline_info);
  }
  


  fun pipeline::make(qvk::engine &engine, qvk::shader vert, qvk::shader frag, metaobj<pipeline> *meta) -> pipeline*
  {
    auto obj = new pipeline(engine, std::move(vert), std::move(frag));

    engine.sub<memory>().push<pipeline>(obj);
    if (meta) meta->connect(obj);
    return obj;
  }

}
