/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#include "mochi/basis.hh"
#include "mochi/except.hh"
#include "mochi/rhi/device.hh"
#include "vulkan/vulkan.hpp"
#include "vulkan/vulkan_raii.hpp"
#include <functional>
#include <string_view>

#define ef else if



namespace mochi::rhi
{

  device::device(
    std::string_view appName, std::array<u16, 4> appVer,
    std::function<i32 (const vk::raii::PhysicalDevices&, PhysicalDeviceSuitable)> GpuPicker,
    vulkan_extension *ext
  )
    : vk_ctx(), vk_inst(nil)
    , vk_phys_dev(nil), vk_device(nil)
    , m_main_q(nil)
    , m_mainPool(nil), m_transferPool(nil), m_transferBuf(nil)
  {
    /// Instance
    l_instance: {
      vk::ApplicationInfo appInfo {
        appName.data(),
        VK_MAKE_API_VERSION(appVer[0], appVer[1], appVer[2], appVer[3]),
        "No Engine",
        VK_MAKE_API_VERSION(0, 1, 0, 0),
        vk::ApiVersion13
      };

      std::vector<const char*> layers {
        #ifdef _mochi_debug_khronos_validator
        "VK_LAYER_KHRONOS_validation",
        #endif
      };
      if (ext && !ext->instance_layers.empty())
        layers.append_range(ext->instance_layers);
      
      std::vector<const char*> extensions {
      };
      if (ext && !ext->instance_extensions.empty())
        extensions.append_range(ext->instance_extensions);

      vk::InstanceCreateInfo createInfo({}, &appInfo, layers, extensions);
      vk_inst = vk::raii::Instance(ctx(), createInfo);
    }



    /// Select Device
    l_select_device: {
      const auto phys_devs = vk::raii::PhysicalDevices(vk_inst);

      if (!GpuPicker | phys_devs.empty())
        throw rhi_error("no GPU was found to start the engine.");
      
      auto phys_idx = GpuPicker(phys_devs, f_is_suitable);
      
      if (phys_idx == -1)
        throw rhi_error("no GPU was found to start the engine.");
      ef (phys_idx <= -2 | phys_idx >= phys_devs.size())
        throw rhi_error("the specified index is out of range.");

      vk_phys_dev = phys_devs[phys_idx];
        
      if (!f_is_suitable(vk_phys_dev))
        throw rhi_error("the selected GPU is not supported by mochi.");
    }



    /// Device
    auto props = vk_phys_dev.getQueueFamilyProperties();
    
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


    std::vector<const char*> extensions = {
      VK_KHR_SWAPCHAIN_EXTENSION_NAME,
      VK_KHR_MAINTENANCE_6_EXTENSION_NAME,
    };
    if (ext && !ext->device_extensions.empty())
      extensions.append_range(ext->device_extensions);


    vk::PhysicalDeviceFeatures features{};
    features.samplerAnisotropy = VK_TRUE;
    features.fillModeNonSolid = VK_TRUE;

    vk::PhysicalDeviceVulkan12Features features12{};
    features12.bufferDeviceAddress = VK_TRUE;
    
    vk::PhysicalDeviceVulkan13Features features13{};
    features13.dynamicRendering = VK_TRUE;
    features13.synchronization2 = VK_TRUE;
    features13.pNext = &features12;


    vk::DeviceCreateInfo dev_info({}, queue_infos, {}, extensions, &features, &features13);
    vk_device = vk::raii::Device(vk_phys_dev, dev_info);


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



  fun device::f_is_suitable(vk::raii::PhysicalDevice phys_dev) -> bool
  {
    auto featureChain = phys_dev.getFeatures2<
      vk::PhysicalDeviceFeatures2, 
      vk::PhysicalDeviceVulkan12Features, 
      vk::PhysicalDeviceVulkan13Features
    >();

    auto features = featureChain.get<vk::PhysicalDeviceFeatures2>();
    auto features12 = featureChain.get<vk::PhysicalDeviceVulkan12Features>();
    auto features13 = featureChain.get<vk::PhysicalDeviceVulkan13Features>();


    return
      features.features.samplerAnisotropy &&
      features.features.fillModeNonSolid &&
      features12.bufferDeviceAddress &&
      features13.dynamicRendering &&
      features13.synchronization2;
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
