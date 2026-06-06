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
#include "mochi/rhi/listPush.hh"
#include "mochi/rhi/listDesc.hh"
#include "mochi/rhi/slotVertex.hh"
#include <vulkan/vulkan.hpp>
#include <vulkan/vulkan_raii.hpp>



namespace mochi::rhi
{

  template<>
  struct info<pipeline>
  {
    public:
      explicit inline info(nil_t): m_push() {}
      explicit info(std::vector<info<listPush>> pushlist, std::vector<info<slotVertex>> vertex, std::vector<info<listDesc>> descset);


    protected:
      std::vector<info<listPush>>   m_push;
      std::vector<info<slotVertex>> m_vertex;
      std::vector<info<listDesc>>    m_descset;

      std::vector<vk::PushConstantRange> vk_PushConstant;
      std::vector<vk::VertexInputBindingDescription> vk_vertexBinding;
      std::vector<vk::VertexInputAttributeDescription> vk_vertexAttribute;
      std::vector<std::vector<vk::DescriptorSetLayoutBinding>> vk_DescBindings;


    public:
      inline fun& push() { return m_push; }
      inline fun& vertex() { return m_vertex; }
      inline fun& desc() { return m_descset; }

      inline fun& pushConstant() { return vk_PushConstant; }
      inline fun  vertexBinding() { return vk::PipelineVertexInputStateCreateInfo({}, vk_vertexBinding, vk_vertexAttribute); }
      inline fun& descBindings() { return vk_DescBindings; }
  };




  struct pipeline
  {
    private:
      explicit pipeline(
        rhi::device &device, info<pipeline> info, std::vector<sptr<shader>> shaders,
        PolygonMode polymode, PrimitiveTopology primitiveTopology,
        Format color_format, Format depth_format
      );
      
    public:
      static inline fun make(
        rhi::device &device, info<pipeline> info, std::vector<sptr<shader>> shaders,
        PolygonMode polymode, PrimitiveTopology primitiveTopology,
        Format color_format, Format depth_format
      ) {
        return make_sptr(new pipeline(device, info, std::move(shaders), polymode, primitiveTopology, color_format, depth_format));
      }
    

    private:
      info<pipeline> m_dbg_info;

      PipelineKind m_kind;
      vk::raii::PipelineLayout vk_layout;
      vk::raii::Pipeline       vk_pipeline;
      std::vector<vk::raii::DescriptorSetLayout> vk_desc_layouts;
      
    public:
      inline fun& info() { return m_dbg_info; }
      inline fun& get() { return vk_pipeline; }
      inline fun  kind() { return m_kind; }
      inline fun& layout() { return vk_layout; }
      inline fun& desc_layouts() { return vk_desc_layouts; }
  };
  
}
