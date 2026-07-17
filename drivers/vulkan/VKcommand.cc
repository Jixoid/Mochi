/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#include "mochi/basis.hh"
#include "drivers/vulkan/VKconvert.hh"
#include "drivers/vulkan/VKcommand.hh"
#include "drivers/vulkan/VKpipeline.hh"
#include "drivers/vulkan/VKrender_target.hh"
#include "mochi/rhi/pipeline.hh"



namespace mochi::rhi::vulkan
{

  fun VK_Command::begin() -> void {
    m_cmd.begin(vk::CommandBufferBeginInfo(vk::CommandBufferUsageFlagBits::eOneTimeSubmit));
  }

  fun VK_Command::end() -> void {
    m_cmd.end();
  }

  fun VK_Command::beginRendering(const RenderTarget &_target, const std::array<float, 4> &clear_color) -> void {
    const auto& target = static_cast<const VK_RenderTarget&>(_target);
    
    vk::ImageMemoryBarrier2 color_barrier(
      vk::PipelineStageFlagBits2::eColorAttachmentOutput,
      vk::AccessFlagBits2::eNone,
      vk::PipelineStageFlagBits2::eColorAttachmentOutput,
      vk::AccessFlagBits2::eColorAttachmentWrite,
      vk::ImageLayout::eUndefined, vk::ImageLayout::eColorAttachmentOptimal,
      VK_QUEUE_FAMILY_IGNORED, VK_QUEUE_FAMILY_IGNORED,
      target.color_image,
      vk::ImageSubresourceRange(vk::ImageAspectFlagBits::eColor, 0, 1, 0, 1)
    );

    std::vector<vk::ImageMemoryBarrier2> barriers;
    barriers.push_back(color_barrier);

    if (target.depth_image) {
      vk::ImageMemoryBarrier2 depth_barrier(
        vk::PipelineStageFlagBits2::eEarlyFragmentTests | vk::PipelineStageFlagBits2::eLateFragmentTests,
        vk::AccessFlagBits2::eNone,
        vk::PipelineStageFlagBits2::eEarlyFragmentTests | vk::PipelineStageFlagBits2::eLateFragmentTests,
        vk::AccessFlagBits2::eDepthStencilAttachmentWrite,
        vk::ImageLayout::eUndefined, vk::ImageLayout::eDepthAttachmentOptimal,
        VK_QUEUE_FAMILY_IGNORED, VK_QUEUE_FAMILY_IGNORED,
        target.depth_image,
        vk::ImageSubresourceRange(vk::ImageAspectFlagBits::eDepth, 0, 1, 0, 1)
      );
      barriers.push_back(depth_barrier);
    }

    m_cmd.pipelineBarrier2(vk::DependencyInfo(vk::DependencyFlagBits::eByRegion, 0, nullptr, 0, nullptr, barriers.size(), barriers.data()));

    vk::ClearValue cv_color;
    cv_color.color = vk::ClearColorValue(std::array<float, 4>{clear_color[0], clear_color[1], clear_color[2], clear_color[3]});

    vk::RenderingAttachmentInfo color_attach(
      target.color_view, vk::ImageLayout::eColorAttachmentOptimal,
      vk::ResolveModeFlagBits::eNone, nil, vk::ImageLayout::eUndefined,
      vk::AttachmentLoadOp::eClear, vk::AttachmentStoreOp::eStore, cv_color
    );

    vk::RenderingInfo rendering_info(
      {}, vk::Rect2D(vk::Offset2D(0, 0), target.extent),
      1, 0,
      1, &color_attach, nullptr, nil
    );


    vk::RenderingAttachmentInfo depth_attach;
    vk::ClearValue cv_depth;
    
    if (target.depth_image) {
      cv_depth.depthStencil = vk::ClearDepthStencilValue(1.0f, 0);
      depth_attach = vk::RenderingAttachmentInfo(
        target.depth_view, vk::ImageLayout::eDepthAttachmentOptimal,
        vk::ResolveModeFlagBits::eNone, nil, vk::ImageLayout::eUndefined,
        vk::AttachmentLoadOp::eClear, vk::AttachmentStoreOp::eDontCare, cv_depth
      );
      rendering_info.setPDepthAttachment(&depth_attach);
    }

    m_cmd.beginRendering(rendering_info);
  }

  fun VK_Command::endRendering(const RenderTarget &_target) -> void {
    const auto& target = static_cast<const VK_RenderTarget&>(_target);
    m_cmd.endRendering();

    vk::ImageMemoryBarrier2 color_barrier(
      vk::PipelineStageFlagBits2::eColorAttachmentOutput,
      vk::AccessFlagBits2::eColorAttachmentWrite,
      vk::PipelineStageFlagBits2::eBottomOfPipe,
      vk::AccessFlagBits2::eNone,
      vk::ImageLayout::eColorAttachmentOptimal, target.final_layout,
      VK_QUEUE_FAMILY_IGNORED, VK_QUEUE_FAMILY_IGNORED,
      target.color_image,
      vk::ImageSubresourceRange(vk::ImageAspectFlagBits::eColor, 0, 1, 0, 1)
    );

    std::vector<vk::ImageMemoryBarrier2> barriers;
    barriers.push_back(color_barrier);

    if (target.depth_image) {
      vk::ImageMemoryBarrier2 depth_barrier(
        vk::PipelineStageFlagBits2::eEarlyFragmentTests | vk::PipelineStageFlagBits2::eLateFragmentTests,
        vk::AccessFlagBits2::eDepthStencilAttachmentWrite,
        vk::PipelineStageFlagBits2::eBottomOfPipe,
        vk::AccessFlagBits2::eNone,
        vk::ImageLayout::eDepthAttachmentOptimal, vk::ImageLayout::eShaderReadOnlyOptimal,
        VK_QUEUE_FAMILY_IGNORED, VK_QUEUE_FAMILY_IGNORED,
        target.depth_image,
        vk::ImageSubresourceRange(vk::ImageAspectFlagBits::eDepth, 0, 1, 0, 1)
      );
      barriers.push_back(depth_barrier);
    }

    m_cmd.pipelineBarrier2(vk::DependencyInfo(vk::DependencyFlagBits::eByRegion, 0, nullptr, 0, nullptr, barriers.size(), barriers.data()));
  }


  fun VK_Command::bindPipeline(Pipeline *pipe) -> void {
    get().bindPipeline(
      VKConvert<PipelineKind>(pipe->kind()),
      static_cast<VK_Pipeline*>(pipe)->get()
    );

    m_pipe = static_cast<VK_Pipeline*>(pipe);
  }

  fun VK_Command::bindDescriptorHeap(sptr<rhi::Buffer> resource_heap, sptr<rhi::Buffer> sampler_heap, u64 resource_size, u64 sampler_size) -> void {
    if (resource_heap) {
      VkBindHeapInfoEXT heap_info = {};
      heap_info.sType = VK_STRUCTURE_TYPE_BIND_HEAP_INFO_EXT;
      heap_info.heapRange.address = resource_heap->address();
      heap_info.heapRange.size = resource_size;
      heap_info.reservedRangeOffset = 0;
      heap_info.reservedRangeSize = 0;

      auto PFN_vkCmdBindResourceHeapEXT = reinterpret_cast<::PFN_vkCmdBindResourceHeapEXT>(vkGetDeviceProcAddr(m_cmd.getDevice(), "vkCmdBindResourceHeapEXT"));
      if (PFN_vkCmdBindResourceHeapEXT) {
        PFN_vkCmdBindResourceHeapEXT(*m_cmd, &heap_info);
      }
    }
    
    if (sampler_heap) {
      VkBindHeapInfoEXT heap_info = {};
      heap_info.sType = VK_STRUCTURE_TYPE_BIND_HEAP_INFO_EXT;
      heap_info.heapRange.address = sampler_heap->address();
      heap_info.heapRange.size = sampler_size;
      heap_info.reservedRangeOffset = 0;
      heap_info.reservedRangeSize = 0;

      auto PFN_vkCmdBindSamplerHeapEXT = reinterpret_cast<::PFN_vkCmdBindSamplerHeapEXT>(vkGetDeviceProcAddr(m_cmd.getDevice(), "vkCmdBindSamplerHeapEXT"));
      if (PFN_vkCmdBindSamplerHeapEXT) {
        PFN_vkCmdBindSamplerHeapEXT(*m_cmd, &heap_info);
      }
    }
  }


  fun VK_Command::pushConstant(ShaderStageFlags stage, u32 off, data dat) -> void {
    vk::PushDataInfoEXT info{
      off,
      vk::HostAddressRangeConstEXT{dat.ptr(), dat.size()}
    };

    get().pushDataEXT(info);
  }

  
  fun VK_Command::draw(offs vertex, offs inst) -> void {
    get().draw(
      vertex.size(), inst.size(),
      vertex.off(),  inst.off()
    );
  }
  
  
  fun VK_Command::dispatch(extent<3,u32> e) -> void {
    get().dispatch(e.x(), e.y(), e.z());
  }


  fun VK_Command::setViewport(u32 first, std::vector<Viewport> viewports) -> void {
    std::vector<vk::Viewport> vps;
    vps.reserve(viewports.size());
    
    for (auto vp: viewports)
      vps.push_back(vk::Viewport(
        vp.min()[0], vp.min()[1],
        vp.max()[0]-vp.min()[0], vp.max()[1]-vp.min()[1],
        vp.min()[2], vp.max()[2]
      ));

    get().setViewport(first, vps);
  }


  fun VK_Command::setScissor(u32 first, std::vector<Rect2D> scissors) -> void {
    std::vector<vk::Rect2D> scs;
    scs.reserve(scissors.size());
    
    for (auto sc: scissors)
      scs.push_back(vk::Rect2D(
        {sc.offset()[0], sc.offset()[1]},
        {sc.extent()[0], sc.extent()[1]}
      ));

    get().setScissor(first, scs);
  }

}
