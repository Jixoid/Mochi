/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#include "mochi/asset/material.hh"
#include "mochi/module/resource.hh"
#include "mochi/module/device.hh"
#include "mochi/rhi/pipeline.hh"



namespace mochi::asset
{

  material::material(core &core, sptr<rhi::pipeline> pip)
    : m_core(core)
    , m_pipeline(std::move(pip))
  {
    m_desc_set = m_core.sub<module::resource>().allocate_descriptor_set(*m_pipeline->desc_layout());
  }

  

  fun material::bind_uniform(u32 binding, const mochi::rhi::buffer &buf) -> void
  {
    vk::DescriptorBufferInfo buffer_info(buf.get(), 0, buf.size());

    vk::WriteDescriptorSet descriptor_write(
      *m_desc_set,
      binding,
      0,
      1,
      vk::DescriptorType::eUniformBuffer,
      nil,
      &buffer_info,
      nil
    );

    m_core.sub<module::device>().vdevice().updateDescriptorSets(descriptor_write, nullptr);
  }

  fun material::bind_texture(u32 binding, vk::ImageView image_view, vk::Sampler sampler) -> void
  {
    vk::DescriptorImageInfo image_info(
      sampler,
      image_view,
      vk::ImageLayout::eShaderReadOnlyOptimal
    );

    vk::WriteDescriptorSet descriptor_write(
      *m_desc_set,
      binding,
      0,
      1,
      vk::DescriptorType::eCombinedImageSampler,
      &image_info,
      nil,
      nil
    );

    m_core.sub<module::device>().vdevice().updateDescriptorSets(descriptor_write, nullptr);
  }

}
