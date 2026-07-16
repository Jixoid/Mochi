/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/

#pragma once

#include <vulkan/vulkan_raii.hpp>
#include "mochi/rhi/pipeline.hh"
#include "mochi/rhi/shader.hh"
#include "vk_mem_alloc.h"



namespace mochi::rhi::vulkan
{

  struct VK_PipelineMeta final: public rhi::PipelineMeta {
    public:
      explicit VK_PipelineMeta(PushConstantList push, VertexBindList vert, DescriptorList desc);

    private:
      std::vector<vk::PushConstantRange> vk_PushConstant;
      std::vector<vk::VertexInputBindingDescription> vk_vertexBinding;
      std::vector<vk::VertexInputAttributeDescription> vk_vertexAttribute;
      std::vector<std::vector<vk::DescriptorSetLayoutBinding>> vk_DescBindings;

    public:
      fun& pushConstant() const { return vk_PushConstant; }
      fun  vertexBinding() const { return vk::PipelineVertexInputStateCreateInfo({}, vk_vertexBinding, vk_vertexAttribute); }
      fun& descBindings() const { return vk_DescBindings; }
  };


  struct VK_Pipeline final: public rhi::Pipeline {
    public:
      explicit VK_Pipeline(
        rhi::DeviceManager &device, PipelineMeta *meta, std::vector<sptr<Shader>> shaders,
        PolygonMode polymode, PrimitiveTopology primitiveTopology,
        Format color_format, Format depth_format
      );

    private:
      vk::raii::PipelineLayout vk_layout;
      vk::raii::Pipeline       vk_pipeline;
      std::vector<vk::raii::DescriptorSetLayout> vk_desc_layouts;
      
    public:
      fun& get() { return vk_pipeline; }
      fun& layout() { return vk_layout; }
      fun& desc_layouts() { return vk_desc_layouts; }
  };

}
