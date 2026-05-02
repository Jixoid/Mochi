#include "qvk/entity/pipeline.hh"
#include "qvk/module/swapchain.hh"
#include "qvk/core.hh"
#include "qvk/shader.hh"
#include "qvk/types.hh"
#include <vulkan/vulkan_raii.hpp>

namespace qvk
{
  fun info<pipeline>::make(core &core, std::vector<qvk::pushSlot> push, std::vector<vertexSlot> vertex, std::vector<uniformSlot> uniform) -> info<pipeline>*
  {
    auto obj = new info<pipeline>(push, vertex, uniform);
    core.sub<memory>().push<info<pipeline>>(obj);
    return obj;
  }

  pipeline::pipeline(core &core, qvk::info<pipeline> *info, std::vector<shaderSlot> shaders)
    : m_info(info), vk_layout(Nil), vk_pipeline(Nil)
  {
    std::vector<vk::PipelineShaderStageCreateInfo> shader_stages;
    shader_stages.reserve(shaders.size());
    
    for (auto &sh: shaders)
      shader_stages.push_back(vk::PipelineShaderStageCreateInfo(
        {}, sh.shaderStage(), sh.shader().module(), sh.shader().entry().c_str()
      ));

    std::array<vk::DynamicState, 2> dynamic_states = { vk::DynamicState::eViewport, vk::DynamicState::eScissor };
    vk::PipelineDynamicStateCreateInfo dynamic_info({}, dynamic_states);

    vk::PipelineInputAssemblyStateCreateInfo input_assembly({}, vk::PrimitiveTopology::eTriangleList, VK_FALSE);
    vk::PipelineViewportStateCreateInfo viewport_state({}, 1, nullptr, 1, nullptr);
    vk::PipelineMultisampleStateCreateInfo multisampling({}, vk::SampleCountFlagBits::e1, VK_FALSE);

    vk::PipelineRasterizationStateCreateInfo rasterizer(
    {}, VK_FALSE, VK_FALSE, vk::PolygonMode::eFill, 
    vk::CullModeFlagBits::eBack, 
    vk::FrontFace::eCounterClockwise, 
    VK_FALSE, 0.0f, 0.0f, 0.0f, 1.0f
    );

    vk::PipelineColorBlendAttachmentState blend_attach(
      VK_FALSE, vk::BlendFactor::eOne, vk::BlendFactor::eZero, vk::BlendOp::eAdd,
      vk::BlendFactor::eOne, vk::BlendFactor::eZero, vk::BlendOp::eAdd,
      vk::ColorComponentFlagBits::eR | vk::ColorComponentFlagBits::eG | vk::ColorComponentFlagBits::eB | vk::ColorComponentFlagBits::eA
    );
    vk::PipelineColorBlendStateCreateInfo color_blending({}, VK_FALSE, vk::LogicOp::eCopy, 1, &blend_attach);

    vk::DescriptorSetLayoutCreateInfo set_info({}, info->vkUniformBindings());
    vk_desc_layout = vk::raii::DescriptorSetLayout(core.sub<device>().vdevice(), set_info);

    vk::DescriptorPoolSize pool_size(vk::DescriptorType::eUniformBuffer, 100);
    vk::DescriptorPoolCreateInfo pool_info(vk::DescriptorPoolCreateFlagBits::eFreeDescriptorSet, 100, pool_size);
    vk_desc_pool = vk::raii::DescriptorPool(core.sub<device>().vdevice(), pool_info);

    vk::PipelineLayoutCreateInfo layout_info({}, *vk_desc_layout, info->vkPushConstant());
    vk_layout = vk::raii::PipelineLayout(core.sub<device>().vdevice(), layout_info);

    // --- DERİNLİK TESTİ AYARLARI ---
    vk::PipelineDepthStencilStateCreateInfo depth_stencil(
      {}, 
      VK_TRUE,                // depthTestEnable
      VK_TRUE,                // depthWriteEnable 
      vk::CompareOp::eLess,   // depthCompareOp
      VK_FALSE,               // depthBoundsTestEnable
      VK_FALSE,               // stencilTestEnable
      {}, {}, 0.0f, 1.0f
    );

    vk::Format depth_format = core.sub<swapchain>().depth_format();
    vk::PipelineRenderingCreateInfo rendering_info(
      0, 1, &core.sub<swapchain>().format(),
      depth_format, vk::Format::eUndefined
    );

    vk::GraphicsPipelineCreateInfo pipeline_info(
      {}, shader_stages, &info->vkVertexInput(), &input_assembly, nullptr,
      &viewport_state, &rasterizer, &multisampling, &depth_stencil,
      &color_blending, &dynamic_info, *vk_layout
    );
    pipeline_info.pNext = &rendering_info;

    vk_pipeline = vk::raii::Pipeline(core.sub<device>().vdevice(), nullptr, pipeline_info);
  }

  fun pipeline::make(core &core, qvk::info<pipeline> *info, std::vector<shaderSlot> shaders) -> pipeline*
  {
    auto obj = new pipeline(core, info, std::move(shaders));
    core.sub<memory>().push<pipeline>(obj);
    return obj;
  }
}