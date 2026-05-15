/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#include "mochi/basis.hh"
#include "mochi/except.hh"
#include "mochi/module/renderer.hh"
#include "mochi/module/device.hh"
#include "vulkan/vulkan.hpp"
#include <vulkan/vulkan_raii.hpp>

#define ef else if



namespace mochi::module
{

  renderer::renderer(device &device)
    : m_device(device)
  {
    m_cmd_buffers = device.getMainBuffer(MAX_FRAMES_IN_FLIGHT);


    vk::SemaphoreCreateInfo sem_info{};
    vk::FenceCreateInfo fence_info(vk::FenceCreateFlagBits::eSignaled);

    for (u32 i{}; i < MAX_FRAMES_IN_FLIGHT; i++) {
      m_image_available_sems.push_back(vk::raii::Semaphore(m_device.get(), sem_info));
      m_in_flight_fences.push_back(vk::raii::Fence(m_device.get(), fence_info));
    }
  }


  

  fun renderer::begin_pass(vk::raii::CommandBuffer &cmd, const rhi::render_target &target, const std::array<float, 4> &clear_color) -> void
  {
    std::vector<vk::ImageMemoryBarrier> barriers;

    barriers.push_back(vk::ImageMemoryBarrier(
      {}, vk::AccessFlagBits::eColorAttachmentWrite,
      vk::ImageLayout::eUndefined, vk::ImageLayout::eColorAttachmentOptimal,
      VK_QUEUE_FAMILY_IGNORED, VK_QUEUE_FAMILY_IGNORED,
      target.color_image, 
      {vk::ImageAspectFlagBits::eColor, 0, 1, 0, 1}
    ));

    if (target.depth_image) {
      barriers.push_back(vk::ImageMemoryBarrier(
        vk::AccessFlagBits::eNone, vk::AccessFlagBits::eDepthStencilAttachmentWrite | vk::AccessFlagBits::eDepthStencilAttachmentRead,
        vk::ImageLayout::eUndefined, vk::ImageLayout::eDepthAttachmentOptimal,
        VK_QUEUE_FAMILY_IGNORED, VK_QUEUE_FAMILY_IGNORED,
        target.depth_image,
        {vk::ImageAspectFlagBits::eDepth, 0, 1, 0, 1}
      ));
    }

    cmd.pipelineBarrier(
      vk::PipelineStageFlagBits::eTopOfPipe, 
      vk::PipelineStageFlagBits::eColorAttachmentOutput | vk::PipelineStageFlagBits::eEarlyFragmentTests,
      {}, {}, {}, barriers
    );


    vk::ClearValue clear_color_val(clear_color);

    vk::RenderingAttachmentInfo color_attachment(
      target.color_view, 
      vk::ImageLayout::eColorAttachmentOptimal,
      vk::ResolveModeFlagBits::eNone, nil, vk::ImageLayout::eUndefined,
      vk::AttachmentLoadOp::eClear, vk::AttachmentStoreOp::eStore, clear_color_val
    );

    vk::RenderingAttachmentInfo depth_attachment;
    vk::RenderingAttachmentInfo* p_depth_attachment = nullptr;
    
    if (target.depth_view) {
      vk::ClearValue clear_depth_val;
      clear_depth_val.depthStencil = vk::ClearDepthStencilValue(1.0f, 0);

      depth_attachment = vk::RenderingAttachmentInfo(
        target.depth_view,
        vk::ImageLayout::eDepthAttachmentOptimal,
        vk::ResolveModeFlagBits::eNone, nil, vk::ImageLayout::eUndefined,
        vk::AttachmentLoadOp::eClear, vk::AttachmentStoreOp::eStore, clear_depth_val
      );
      p_depth_attachment = &depth_attachment;
    }

    vk::RenderingInfo render_info(
      {}, {{0, 0}, target.extent}, 
      1, 0, 1, &color_attachment, p_depth_attachment, nil
    );
    cmd.beginRendering(render_info);
  }

  fun renderer::end_pass(vk::raii::CommandBuffer &cmd, const rhi::render_target &target) -> void
  {
    cmd.endRendering();

    // Transition color attachment to final layout
    vk::ImageMemoryBarrier img_barrier(
      vk::AccessFlagBits::eColorAttachmentWrite, {},
      vk::ImageLayout::eColorAttachmentOptimal, target.final_layout,
      VK_QUEUE_FAMILY_IGNORED, VK_QUEUE_FAMILY_IGNORED,
      target.color_image, 
      {vk::ImageAspectFlagBits::eColor, 0, 1, 0, 1}
    );
    
    cmd.pipelineBarrier(
      vk::PipelineStageFlagBits::eColorAttachmentOutput, vk::PipelineStageFlagBits::eBottomOfPipe,
      {}, {}, {}, {img_barrier}
    );
  }



  fun renderer::begin_frame() -> vk::raii::CommandBuffer&
  {
    vk::Result err = m_device.vdevice().waitForFences({*m_in_flight_fences[m_current_frame]}, VK_TRUE, UINT64_MAX);

    if (err != vk::Result::eSuccess)
      throw mochi::rhi_error("Error waiting for GPU or device lost!");
    

    m_device.vdevice().resetFences({*m_in_flight_fences[m_current_frame]});


    vk::raii::CommandBuffer &cmd = m_cmd_buffers[m_current_frame];
    cmd.reset();
    cmd.begin({vk::CommandBufferUsageFlagBits::eOneTimeSubmit});

    return cmd;
  }

  fun renderer::end_frame(vk::raii::CommandBuffer &cmd, std::span<vk::Semaphore> wait_sems, std::span<vk::Semaphore> signal_sems) -> void
  {
    cmd.end();


    std::vector<vk::PipelineStageFlags> wait_stages(wait_sems.size(), vk::PipelineStageFlagBits::eColorAttachmentOutput);
    
    vk::SubmitInfo submit_info(
      wait_sems.size(), wait_sems.data(), wait_stages.data(),
      1, &*cmd,
      signal_sems.size(), signal_sems.data()
    );

    m_device.graphics_q().best().get().submit(submit_info, *m_in_flight_fences[m_current_frame]);


    m_current_frame = (m_current_frame + 1) % MAX_FRAMES_IN_FLIGHT;
  }

}
