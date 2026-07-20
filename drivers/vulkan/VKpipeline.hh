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
#include "mochi/rhi/utility/pipeline_cache_utility.hh"
#include "mochi/rhi/shader.hh"
#include "vk_mem_alloc.h"



namespace mochi::rhi::vulkan
{

  struct VK_PipelineMeta final: public rhi::PipelineMeta {
    public:
      explicit VK_PipelineMeta(PushConstantList push, VertexBindList vert);

    private:
      std::vector<vk::PushConstantRange> vk_PushConstant;
      std::vector<vk::VertexInputBindingDescription> vk_vertexBinding;
      std::vector<vk::VertexInputAttributeDescription> vk_vertexAttribute;
      u32 m_push_data_end{0}; // byte offset right after last field (before struct padding)

    public:
      fun& pushConstant() const { return vk_PushConstant; }
      fun  vertexBinding() const { return vk::PipelineVertexInputStateCreateInfo({}, vk_vertexBinding, vk_vertexAttribute); }
      fun  push_data_end() const { return m_push_data_end; }
  };


  struct VK_Pipeline final: public rhi::Pipeline {
    public:
      explicit VK_Pipeline(
        rhi::Device &device, rhi::PipelineCacheUtility &pmng, u64 sign,
        PipelineMeta *meta, std::vector<sptr<Shader>> shaders,
        PolygonMode polymode, PrimitiveTopology primitiveTopology,
        Format color_format, Format depth_format
      );

    private:
      vk::raii::PipelineCache  vk_pipelineCache;
      vk::raii::PipelineLayout vk_layout;
      vk::raii::Pipeline       vk_pipeline;
      
    public:
      fun& get() { return vk_pipeline; }
      fun& layout() { return vk_layout; }
  };

}
