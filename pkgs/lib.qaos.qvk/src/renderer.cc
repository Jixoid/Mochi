/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#include "qvk/renderer.hh"
#include "qvk/device.hh"
#include <iostream>
#include <vulkan/vulkan_raii.hpp>

#define ef else if



namespace qvk
{

  renderer::renderer(device &device, swapchain &swapchain)
    : m_device(device), m_swapchain(swapchain), m_cmd_pool(Nil)
  {
    // Command Pool
    vk::CommandPoolCreateInfo pool_info(
      vk::CommandPoolCreateFlagBits::eResetCommandBuffer, 
      0 // Not: Buraya cihazından bulduğun grafik kuyruğu indisini vermelisin
    );
    m_cmd_pool = vk::raii::CommandPool(m_device.vdevice(), pool_info);

    
    // Command Buffer 
    auto image_count = m_swapchain.image_count();

    vk::CommandBufferAllocateInfo alloc_info(*m_cmd_pool, vk::CommandBufferLevel::ePrimary, MAX_FRAMES_IN_FLIGHT);
    m_cmd_buffers = vk::raii::CommandBuffers(m_device.vdevice(), alloc_info);



    // Synchronize
    vk::SemaphoreCreateInfo sem_info{};
    vk::FenceCreateInfo fence_info(vk::FenceCreateFlagBits::eSignaled);

    for (u32 i{}; i < MAX_FRAMES_IN_FLIGHT; i++) {
      m_image_available_sems.push_back(vk::raii::Semaphore(m_device.vdevice(), sem_info));
      m_in_flight_fences.push_back(vk::raii::Fence(m_device.vdevice(), fence_info));
    }

    for (u32 i{}; i < image_count; i++) {
      m_render_finished_sems.push_back(vk::raii::Semaphore(m_device.vdevice(), sem_info));
    }
  }


  

  fun renderer::begin_swapchain_rendering(vk::raii::CommandBuffer &cmd, const std::array<float, 4> &clear_color) -> void
  {
    // 1. Resmi Çizime Hazırla (Undefined -> ColorAttachment)
    vk::ImageMemoryBarrier img_barrier(
      {}, vk::AccessFlagBits::eColorAttachmentWrite,
      vk::ImageLayout::eUndefined, vk::ImageLayout::eColorAttachmentOptimal,
      VK_QUEUE_FAMILY_IGNORED, VK_QUEUE_FAMILY_IGNORED,
      m_swapchain.images()[m_image_index], // Artık Swapchain'e doğrudan Renderer erişiyor
      {vk::ImageAspectFlagBits::eColor, 0, 1, 0, 1}
    );
    
    cmd.pipelineBarrier(
      vk::PipelineStageFlagBits::eTopOfPipe, vk::PipelineStageFlagBits::eColorAttachmentOutput,
      {}, {}, {}, {img_barrier}
    );

    // 2. Dynamic Rendering Başlat
    vk::ClearValue clear_val(clear_color);
    vk::RenderingAttachmentInfo color_attachment(
      *m_swapchain.image_views()[m_image_index], 
      vk::ImageLayout::eColorAttachmentOptimal,
      vk::ResolveModeFlagBits::eNone, nullptr, vk::ImageLayout::eUndefined,
      vk::AttachmentLoadOp::eClear, vk::AttachmentStoreOp::eStore, clear_val
    );
    
    vk::RenderingInfo render_info(
      {}, {{0, 0}, m_swapchain.extent()}, 
      1, 0, 1, &color_attachment
    );

    cmd.beginRendering(render_info);
  }

  fun renderer::end_swapchain_rendering(vk::raii::CommandBuffer &cmd) -> void
  {
    // 1. Dynamic Rendering Bitir
    cmd.endRendering();

    // 2. Resmi Ekrana Hazırla (ColorAttachment -> PresentSrc)
    vk::ImageMemoryBarrier img_barrier(
      vk::AccessFlagBits::eColorAttachmentWrite, {},
      vk::ImageLayout::eColorAttachmentOptimal, vk::ImageLayout::ePresentSrcKHR,
      VK_QUEUE_FAMILY_IGNORED, VK_QUEUE_FAMILY_IGNORED,
      m_swapchain.images()[m_image_index], 
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
      throw std::runtime_error("GPU beklenirken hata oluştu veya cihaz kaybedildi!");
    

    // Request a new image from Swapchain
    auto [result, img_idx] = m_swapchain.get().acquireNextImage(
      UINT64_MAX, 
      *m_image_available_sems[m_current_frame], 
      nullptr
    );
    m_image_index = img_idx;


    // Close the fence
    m_device.vdevice().resetFences({*m_in_flight_fences[m_current_frame]});


    // Reset the command prompt and start typing
    vk::raii::CommandBuffer& cmd = m_cmd_buffers[m_current_frame];
    cmd.reset();
    cmd.begin({vk::CommandBufferUsageFlagBits::eOneTimeSubmit});

    return cmd;
  }

  fun renderer::end_frame(vk::raii::CommandBuffer &cmd) -> void
  {
    cmd.end();

    // GPU Submit
    vk::PipelineStageFlags wait_stage = vk::PipelineStageFlagBits::eColorAttachmentOutput;
    
    vk::SubmitInfo submit_info(
      1, &*m_image_available_sems[m_current_frame], &wait_stage,
      1, &*cmd,
      1, &*m_render_finished_sems[m_image_index]
    );

    m_device.graphics_q().best().submit(submit_info, *m_in_flight_fences[m_current_frame]);


    // Screen Present
    vk::PresentInfoKHR present_info(
      1, &*m_render_finished_sems[m_image_index],
      1, &*m_swapchain.get(),
      &m_image_index
    );


    // Send Output
    auto err = m_device.graphics_q().best().presentKHR(present_info);

    if (err == vk::Result::eErrorOutOfDateKHR || err == vk::Result::eSuboptimalKHR)
    {
      // Burada pencere boyutunun değiştiğini anlıyoruz.
      // Örn: m_swapchain.recreate(); 
      std::cout << "Pencere boyutu degisti, Swapchain yenilenmeli!" << std::endl;
    }
    ef (err != vk::Result::eSuccess)
      throw std::runtime_error("Ekrana goruntu basilirken kritik hata olustu!");


    // Next Frame
    m_current_frame = (m_current_frame + 1) % MAX_FRAMES_IN_FLIGHT;
  }

}
