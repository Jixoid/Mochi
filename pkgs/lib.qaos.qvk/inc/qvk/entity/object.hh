/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#pragma once

#include "Basis.hh"
#include "qvk/types.hh"
#include <vulkan/vulkan_raii.hpp>



namespace qvk
{

  struct vertexInstSlot
  {
    public:
      inline vertexInstSlot(buffer *buf, vk::VertexInputRate inputRate)
        : m_buf(buf)
        , m_inputRate(inputRate)
      {}


    private:
      buffer *m_buf;
      vk::VertexInputRate m_inputRate;

    public:
      inline fun buf() { return m_buf; }
      inline fun inputRate() { return m_inputRate; }
  };

  struct uniformInstSlot
  {
    public:
      inline uniformInstSlot(buffer *buf, vk::ShaderStageFlags shaderStage)
        : m_buf(buf)
        , m_shaderStage(shaderStage)
      {}


    private:
      buffer *m_buf;
      vk::ShaderStageFlags m_shaderStage;

    public:
      inline fun buf() { return m_buf; }
      inline fun shaderStage() { return m_shaderStage; }

  };




  struct object
  {
    public:
      explicit inline object(pipeline *pipeline, std::vector<vertexInstSlot> vertexs, std::vector<uniformInstSlot> uniforms, u32 vertex_count, u32 instance_count)
        : m_pipeline(pipeline)
        , m_vertexs(vertexs)
        , m_uniforms(uniforms)
        , m_vertex_count(vertex_count)
        , m_instance_count(instance_count)
      {}

    public:
      static fun make(core &core, pipeline *pipeline, std::vector<vertexInstSlot> vertexs, std::vector<uniformInstSlot> uniforms, u32 vertex_count, u32 instance_count) -> object*;


    private:
      pipeline *m_pipeline;
      std::vector<vertexInstSlot> m_vertexs;
      std::vector<uniformInstSlot> m_uniforms;
      u32 m_vertex_count;
      u32 m_instance_count;
      vk::raii::DescriptorSets m_desc_sets{nullptr};

    public:
      inline fun  pipeline() { return m_pipeline; }
      inline fun& vertexs() { return m_vertexs; }
      inline fun& uniforms() { return m_uniforms; }
      inline fun  vertex_count() { return m_vertex_count; }
      inline fun  instance_count() { return m_instance_count; }
      inline fun& desc_sets() { return m_desc_sets; }
  };

}
