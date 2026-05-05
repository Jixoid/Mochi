/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#include "Basis.hh"
#include "qvk/entity/visual.hh"
#include "qvk/entity/camera.hh"
#include "qvk/entity/mesh.hh"
#include "qvk/entity/node.hh"
#include "qvk/entity/node.hh"
#include "qvk/entity/pipeline.hh"
#include "qvk/core.hh"
#include "qvk/types.hh"
#include <vulkan/vulkan_raii.hpp>
#include <vulkan/vulkan.h>



namespace qvk
{
  
  info<pipeline> pbr_i(
    {
      {gt::make<mat4<f32>>(), vk::ShaderStageFlagBits::eVertex | vk::ShaderStageFlagBits::eFragment},
    },
    {
      {&vertex_i, vk::VertexInputRate::eVertex},
    },
    {
      {&camera_i, vk::DescriptorType::eUniformBuffer, vk::ShaderStageFlagBits::eVertex | vk::ShaderStageFlagBits::eFragment},
    }
  );




  visual::visual(core &core, node *parent, mat4<f32> model, mesh *mesh, pipeline *pipeline)
    : node(parent, model)
    , m_mesh(mesh)
    , m_pipeline(pipeline)
  {
    vk::DescriptorSetAllocateInfo alloc_info(*m_pipeline->desc_pool(), *m_pipeline->desc_layout());
    m_desc_sets = vk::raii::DescriptorSets(core.sub<device>().vdevice(), alloc_info);

    // 2. Memory modülünden güncel Kamera UBO'sunu çek
    buffer* cam_ubo = core.sub<memory>().camera_ubo();
    buffer* lig_ubo = core.sub<memory>().light_ubo();

    // 3. GPU'ya "Benim descriptor set'imin 0. slotuna bu buffer'ı bağla" komutunu hazırla
    vk::DescriptorBufferInfo cam_buffer_info(*cam_ubo->get(), 0, camera_i.stride());
    vk::DescriptorBufferInfo lig_buffer_info(*lig_ubo->get(), 0, lig_ubo->size());

    std::vector<vk::WriteDescriptorSet> writes;

    writes.push_back(vk::WriteDescriptorSet(
      *m_desc_sets[0], 
      0, // Binding Index (pbr_i'deki sıraya göre 0. binding kamera UBO'su)
      0, 1, 
      vk::DescriptorType::eUniformBuffer, 
      nullptr, 
      &cam_buffer_info, 
      nullptr
    ));

    writes.push_back(vk::WriteDescriptorSet(
      *m_desc_sets[0], 
      1, // Binding Index (pbr_i'deki sıraya göre 0. binding kamera UBO'su)
      0, 1, 
      vk::DescriptorType::eUniformBuffer, 
      nullptr, 
      &lig_buffer_info, 
      nullptr
    ));

    // 4. Bağlantıyı GPU'ya yaz
    core.sub<device>().vdevice().updateDescriptorSets(writes, nullptr);
  }


  fun visual::make(core &core, node *parent, mat4<f32> model, mesh *mesh, pipeline *pipeline) -> visual*
  {
    auto obj = new visual(core, parent, model, mesh, pipeline);

    core.sub<memory>().push<visual>(obj);
    return obj;
  }

}
