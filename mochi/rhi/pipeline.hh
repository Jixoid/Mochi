/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#pragma once

#include "mochi/basis.hh"
#include "mochi/rhi/rhi.hh"
#include "mochi/types.hh"
#include "mochi/rhi/shader.hh"
#include "mochi/rhi/slotPush.hh"
#include "mochi/rhi/slotVertex.hh"
#include <vulkan/vulkan.hpp>
#include <vulkan/vulkan_raii.hpp>



namespace mochi::rhi
{

  template<>
  struct info<pipeline>
  {
    private:
      explicit info(std::vector<sptr<info<slotPush>>> push, std::vector<sptr<info<slotVertex>>> vertex, std::vector<sptr<info<descset>>> descset);

    public:
      static inline fun make(std::vector<sptr<info<slotPush>>> push, std::vector<sptr<info<slotVertex>>> vertex, std::vector<sptr<info<descset>>> descset) {
        return make_sptr(new info<pipeline>(std::move(push), std::move(vertex), std::move(descset)));
      }


    protected:
      info() = default;
    public:
      virtual ~info() = default;


    protected:
      std::vector<sptr<info<slotPush>>>   m_push;
      std::vector<sptr<info<slotVertex>>> m_vertex;
      std::vector<sptr<info<descset>>>    m_descset;

      std::vector<vk::PushConstantRange> vk_PushConstant;
      std::vector<vk::VertexInputBindingDescription> vk_vertexBinding;
      std::vector<vk::VertexInputAttributeDescription> vk_vertexAttribute;
      std::vector<std::vector<vk::DescriptorSetLayoutBinding>> vk_DescBindings;


    public:
      inline const fun& push() { return m_push; }
      inline const fun& vertex() { return m_vertex; }
      inline const fun& descset() { return m_descset; }

      inline fun& pushConstant() { return vk_PushConstant; }
      inline fun  vertexBinding() { return vk::PipelineVertexInputStateCreateInfo({}, vk_vertexBinding, vk_vertexAttribute); }
      inline fun& descBindings() { return vk_DescBindings; }
  };




  struct pipeline
  {
    private:
      explicit pipeline(
        module::device &device, sptr<info<pipeline>> info, std::vector<sptr<shader>> shaders,
        PolygonMode polymode, PrimitiveTopology primitiveTopology,
        Format color_format, Format depth_format
      );
      
    public:
      static inline fun make(
        module::device &device, sptr<info<pipeline>> info, std::vector<sptr<shader>> shaders,
        PolygonMode polymode, PrimitiveTopology primitiveTopology,
        Format color_format, Format depth_format
      ) {
        return make_sptr(new pipeline(device, info, std::move(shaders), polymode, primitiveTopology, color_format, depth_format));
      }
    

    private:
      sptr<info<pipeline>>     m_info{};
      vk::raii::PipelineLayout vk_layout;
      vk::raii::Pipeline       vk_pipeline;
      std::vector<vk::raii::DescriptorSetLayout> vk_desc_layouts;
      
    public:
      inline fun  info() { return m_info.get(); }
      inline fun& get() { return vk_pipeline; }
      inline fun& layout() { return vk_layout; }
      inline fun& desc_layouts() { return vk_desc_layouts; }
  };
  
}
