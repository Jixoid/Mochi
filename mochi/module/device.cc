/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#include "mochi/module/device.hh"
#include "vulkan/vulkan.hpp"
#include <vulkan/vulkan_raii.hpp>



namespace mochi::module
{

  device::device(vk::raii::PhysicalDevice phys_dev)
    : vk_phys_dev(phys_dev), vk_device(nil)
    , m_main_q(nil)
    , m_mainPool(nil), m_transferPool(nil), m_transferBuf(nil)
  {
    // Allocate one queue per available queue family with default priority
    auto props = phys_dev.getQueueFamilyProperties();
    
    std::vector<std::vector<f32>> all_priorities(props.size());
    std::vector<vk::DeviceQueueCreateInfo> queue_infos;
    queue_infos.reserve(props.size());

    for (u32 i{}; i < props.size(); i++) {
      u32 count = props[i].queueCount;

      all_priorities[i].resize(count, 1);

      queue_infos.push_back({
        {}, i, count, all_priorities[i].data()
      });
    }


    std::vector<const char*> extensions = { VK_KHR_SWAPCHAIN_EXTENSION_NAME };

    vk::PhysicalDeviceFeatures features;
    features.samplerAnisotropy = VK_TRUE;
    features.fillModeNonSolid = VK_TRUE;

    vk::PhysicalDeviceVulkan12Features features12{};
    features12.bufferDeviceAddress = VK_TRUE;
    
    vk::PhysicalDeviceDynamicRenderingFeatures dynamicRenderingFeature;
    dynamicRenderingFeature.dynamicRendering = VK_TRUE;
    dynamicRenderingFeature.pNext = features12;


    vk::DeviceCreateInfo dev_info({}, queue_infos, {}, extensions, &features, &dynamicRenderingFeature);
    vk_device = vk::raii::Device(phys_dev, dev_info);


    for (u32 i{}; i < props.size(); i++)
    {
      auto flags = props[i].queueFlags;
      u32 count = props[i].queueCount;

      for (u32 q_idx{}; q_idx < count; q_idx++)
      {
        // MAIN
        if (m_main_q.family() == -1 && flags & (vk::QueueFlagBits::eGraphics | vk::QueueFlagBits::eCompute))
          m_main_q = {vk::raii::Queue(vk_device, i, q_idx), i};


        // GRAPHIC
        if (flags & vk::QueueFlagBits::eGraphics)
          graphics_q().m_primary.push_back({vk::raii::Queue(vk_device, i, q_idx), i});
        

        // COMPUTE
        if (flags & vk::QueueFlagBits::eCompute) {
          if (!(flags & vk::QueueFlagBits::eGraphics))
            compute_q().m_primary.push_back({vk::raii::Queue(vk_device, i, q_idx), i});
          else
            compute_q().m_secondary.push_back({vk::raii::Queue(vk_device, i, q_idx), i});
        }


        // TRANSFER
        if (flags & vk::QueueFlagBits::eTransfer) {
          if (!(flags & vk::QueueFlagBits::eGraphics) && !(flags & vk::QueueFlagBits::eCompute))
            transfer_q().m_primary.push_back({vk::raii::Queue(vk_device, i, q_idx), i});
          else
            transfer_q().m_secondary.push_back({vk::raii::Queue(vk_device, i, q_idx), i});
        }
      }
    }



    vk::CommandPoolCreateInfo main_pool_info(vk::CommandPoolCreateFlagBits::eResetCommandBuffer, main_q().family());
    m_mainPool = vk::raii::CommandPool(vk_device, main_pool_info);

    vk::CommandPoolCreateInfo transfer_pool_info(vk::CommandPoolCreateFlagBits::eResetCommandBuffer, transfer_q().best().family());
    m_transferPool = vk::raii::CommandPool(vk_device, transfer_pool_info);


    vk::CommandBufferAllocateInfo alloc_info(*m_transferPool, vk::CommandBufferLevel::ePrimary, 1);
    m_transferBuf = std::move(vk::raii::CommandBuffers(vk_device, alloc_info).front());
  }




  fun device::getMainBuffer(u32 count) -> vk::raii::CommandBuffers
  {
    vk::CommandBufferAllocateInfo alloc_info(*m_mainPool, vk::CommandBufferLevel::ePrimary, count);
    return vk::raii::CommandBuffers(vk_device, alloc_info);
  }

  fun device::getTransferBuffer(u32 count) -> vk::raii::CommandBuffers
  {
    vk::CommandBufferAllocateInfo alloc_info(*m_transferPool, vk::CommandBufferLevel::ePrimary, count);
    return vk::raii::CommandBuffers(vk_device, alloc_info);
  }



  fun device::flushTransferBuf() -> void
  {
    if (m_transferBuf_used) {
      auto &cmd = m_transferBuf;

      cmd.end();

      vk::SubmitInfo submit_info({}, {}, *cmd, {});
      transfer_q().best().get().submit(submit_info, nil);
      transfer_q().best().get().waitIdle();

      m_transferBuf_refs.clear();
      m_transferBuf_used = false;
    }
  }

}
