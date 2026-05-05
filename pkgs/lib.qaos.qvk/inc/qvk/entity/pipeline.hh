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
#include "qvk/shader.hh"
#include "vulkan/vulkan.hpp"
#include <vulkan/vulkan_raii.hpp>
#include <vulkan/vulkan.hpp>



namespace qvk
{

  struct pushSlot
  {
    public:
      inline pushSlot(gt type, vk::ShaderStageFlags shaderStage)
        : m_type(type)
        , m_shaderStage(shaderStage)
      {}


    private:
      gt m_type;
      vk::ShaderStageFlags m_shaderStage;

    public:
      inline fun type() { return m_type; }
      inline fun shaderStage() { return m_shaderStage; }

  };


  struct vertexSlot
  {
    public:
      inline vertexSlot(info<buffer> *ibuf, vk::VertexInputRate inputRate)
        : m_ibuf(ibuf)
        , m_inputRate(inputRate)
      {}


    private:
      info<buffer> *m_ibuf;
      vk::VertexInputRate m_inputRate;

    public:
      inline fun ibuf() { return m_ibuf; }
      inline fun inputRate() { return m_inputRate; }

  };


  struct descriptorSlot
  {
    public:
      inline descriptorSlot(info<buffer> *ibuf, vk::DescriptorType kind, vk::ShaderStageFlags shaderStage)
        : m_ibuf(ibuf)
        , m_kind(kind)
        , m_shaderStage(shaderStage)
      {}

      
    private:
      info<buffer> *m_ibuf;
      vk::DescriptorType m_kind;
      vk::ShaderStageFlags m_shaderStage;

    public:
      inline fun ibuf() { return m_ibuf; }
      inline fun kind() { return m_kind; }
      inline fun shaderStage() { return m_shaderStage; }
  };


  struct shaderSlot
  {
    public:
      inline shaderSlot(vk::ShaderStageFlagBits shaderStage, shader shader)
        : m_shaderStage(shaderStage)
        , m_shader(std::move(shader))
      {}


    private:
      vk::ShaderStageFlagBits m_shaderStage;
      shader m_shader;

    public:
      inline fun& shader() { return m_shader; }
      inline fun  shaderStage() { return m_shaderStage; }

  };





  template<>
  struct info<pipeline>
  {
    public:
      explicit info(std::vector<pushSlot> push, std::vector<vertexSlot> vertex, std::vector<descriptorSlot> descriptor);

    private:
      std::vector<pushSlot> m_push;
      std::vector<vertexSlot> m_vertex;
      std::vector<descriptorSlot> m_descriptor;

      std::vector<vk::VertexInputBindingDescription> vk_vib;
      std::vector<vk::VertexInputAttributeDescription> vk_via;

      std::vector<vk::PushConstantRange> vk_PushConstant;
      std::vector<vk::DescriptorSetLayoutBinding> vk_DescriptorBindings;
      

    public:
      inline fun push() { return m_push; }
      inline fun vertex() { return m_vertex; }
      inline fun descriptor() { return m_descriptor; }

      inline fun& vkPushConstant() { return vk_PushConstant; }
      inline fun  vkVertexInput() { return vk::PipelineVertexInputStateCreateInfo({}, vk_vib, vk_via); }
      inline fun& vkDescriptorBindings() { return vk_DescriptorBindings; }
  };




  struct pipeline
  {
    private:
      explicit pipeline(core &core, info<pipeline> *info, std::vector<shaderSlot> shaders);

    public:
      static fun make(core &core, info<pipeline> *info, std::vector<shaderSlot> shaders) -> pipeline*;
    

    private:
      info<pipeline> *m_info{};
      vk::raii::PipelineLayout vk_layout;
      vk::raii::Pipeline       vk_pipeline;
      vk::raii::DescriptorSetLayout vk_desc_layout{nullptr};
      vk::raii::DescriptorPool      vk_desc_pool{nullptr};

    public:
      inline fun info() { return m_info; }
      inline fun& get() { return vk_pipeline; }
      inline fun& layout() { return vk_layout; }
      inline fun& desc_layout() { return vk_desc_layout; }
      inline fun& desc_pool() { return vk_desc_pool; }
  };
  
}
