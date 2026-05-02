/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#include "Basis.hh"
#include "qvk/entity/pipeline.hh"
#include "qvk/module/memory.hh"
#include "qvk/entity/object.hh"
#include "qvk/core.hh"
#include <vulkan/vulkan_raii.hpp>



namespace qvk
{

  fun object::make(core &core, qvk::pipeline *pipeline, std::vector<vertexInstSlot> vertexs, std::vector<uniformInstSlot> uniforms, u32 vertex_count, u32 instance_count) -> object*
  {
    auto obj = new object(pipeline, vertexs, uniforms, vertex_count, instance_count);

    // --- YENİ: DESCRIPTOR SET (PAKET) TAHSİSİ VE YAZIMI ---
    if (!uniforms.empty()) {
      // 1. Havuzdan Set Kopar
      vk::DescriptorSetAllocateInfo alloc_info(*pipeline->desc_pool(), *pipeline->desc_layout());
      obj->m_desc_sets = vk::raii::DescriptorSets(core.sub<device>().vdevice(), alloc_info);

      // 2. Buffer verilerini Set'e yazdır
      std::vector<vk::DescriptorBufferInfo> buffer_infos(uniforms.size());
      std::vector<vk::WriteDescriptorSet> writes(uniforms.size());

      for (u32 i = 0; i < uniforms.size(); i++) {
        buffer_infos[i] = vk::DescriptorBufferInfo(*uniforms[i].buf()->get(), 0, uniforms[i].buf()->size());
        
        writes[i] = vk::WriteDescriptorSet(
          *obj->m_desc_sets[0], i, 0, 1, 
          vk::DescriptorType::eUniformBuffer, 
          nullptr, &buffer_infos[i], nullptr
        );
      }
      
      // 3. GPU'ya güncellemeyi yolla
      core.sub<device>().vdevice().updateDescriptorSets(writes, nullptr);
    }

    core.sub<memory>().push<object>(obj);
    return obj;
  }

}
