/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#include "Basis.h"
#include "Basis.hh"
#include "qvk/entity/pipeline.hh"
#include "qvk/module/memory.hh"
#include "qvk/module/meta.hh"
#include "qvk/types.hh"
#include "vulkan/vulkan.hpp"
#include <cassert>
#include <format>
#include <string>
#include <vector>
#include <vulkan/vulkan_raii.hpp>
#include <vulkan/vulkan.h>



namespace qvk
{

  inline fun getSize(u0 __size) -> std::string
  {
    f32 size = __size;

    std::vector<std::string> typs = {"B", "KB", "MB", "GB", "TB", "PB"};
    int idx{};

    while (size >= 1024) {
      idx++;
      size /= 1024;
    }

    return std::format("{}{}", size, typs[idx]);
  }

  

  meta::meta(qvk::memory &memory)
    : m_memory(memory)
  {}



  constexpr inline fun align_size(u64 size, u64 alignment) { if (alignment == 0) return size; else return ((size + alignment - 1) / alignment) * alignment; }


  fun meta::compile() -> void
  {
    /// Pipeline
    for (auto pipe: m_memory.list<info<pipeline>>())
    {
      /// Push Constant
      std::vector<vk::PushConstantRange> pcr;
      u64 off{};
      
      for (auto typ: pipe->push()) {
        off = align_size(off, typ.type().align());

        pcr.push_back(vk::PushConstantRange(
          typ.shaderStage(),
          off,
          typ.type().size() * typ.type().count()
        ));

        off += typ.type().size();
      }
      pipe->vkPushConstant() = pcr;
  


      /// Vertex Buffers
      auto &vib = pipe->vk_vib; vib.clear();
      auto &via = pipe->vk_via; via.clear();
      u32 binding_idx{}, location_idx{};

      for (auto ibuf: pipe->vertex()) {
        u32 v_offset{};

        for (auto &typ: ibuf.ibuf()->items()) {
          v_offset = align_size(v_offset, typ.align());
          
          for (u32 c{}; c < typ.count(); c++) {
            via.push_back(vk::VertexInputAttributeDescription(
              location_idx++,
              binding_idx,
              typ.format(),
              v_offset
            ));
            
            v_offset += typ.size();
          }
        }

        vib.push_back(vk::VertexInputBindingDescription(
          binding_idx++,
          ibuf.ibuf()->stride(),
          ibuf.inputRate()
        )); 
      }
      pipe->vkVertexInput() = vk::PipelineVertexInputStateCreateInfo({}, vib, via);
    
      
    
      /// Uniform Buffers (Descriptor Set Layout Bindings)
      std::vector<vk::DescriptorSetLayoutBinding> bindings;
      u32 ubo_binding_idx{};

      for (auto ubo: pipe->uniform()) {
        bindings.push_back(vk::DescriptorSetLayoutBinding(
          ubo_binding_idx++,                   // Binding Numarası (0, 1, 2...)
          vk::DescriptorType::eUniformBuffer,  // Vulkan'a bunun bir UBO olduğunu söylüyoruz
          1,                                   // Descriptor Sayısı (Genelde 1 olur)
          ubo.shaderStage(),                   // Hangi Shader aşaması erişecek?
          nullptr
        ));
      }
      pipe->vkUniformBindings() = bindings;
    }



  }

}
