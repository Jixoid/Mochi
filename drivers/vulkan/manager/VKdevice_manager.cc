/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#include "drivers/vulkan/manager/VKdevice_manager.hh"
#include "drivers/vulkan/VKdriver.hh"
#include "drivers/vulkan/VKimage.hh"
#include "drivers/vulkan/VKsampler.hh"
#include "mochi/debug/debug.hh"
#include "mochi/rhi/manager/alloc_manager.hh"
#include "mochi/rhi/image.hh"
#include "mochi/rhi/sampler.hh"
#include "drivers/vulkan/VKsampler.hh"
#include "mochi/basis.hh"
#include "mochi/except.hh"
#include "vulkan/vulkan.hpp"
#include "vulkan/vulkan_raii.hpp"
#include <format>
#include <string_view>
#include <vulkan/vulkan_core.h>
#include <GLFW/glfw3.h>

#define ef else if



namespace mochi::rhi::vulkan::mng
{

  extern "C" fun MochiRHI_MakeDeviceManager(std::string_view appName, std::array<u16, 4> appVer) -> rhi::mng::DeviceManager* {
    return new rhi::vulkan::mng::VK_DeviceManager(appName, appVer);
  }


  VK_DeviceManager::VK_DeviceManager(std::string_view appName, std::array<u16, 4> appVer)
    : rhi::mng::DeviceManager()
    , vk_ctx(), vk_inst(nil)
    , vk_phys_dev(nil), vk_device(nil)
    , m_mainPool(nil)
  {
    /// Instance
    l_instance: {
      vk::ApplicationInfo appInfo {
        appName.data(),
        VK_MAKE_API_VERSION(appVer[0], appVer[1], appVer[2], appVer[3]),
        "No Engine",
        VK_MAKE_API_VERSION(0, 1, 0, 0),
        vk::ApiVersion14
      };

      std::vector<const char*> layers {
        #ifdef _mochi_debug_khronos_validator
        "VK_LAYER_KHRONOS_validation",
        #endif
      };
      //if (ext && !ext->instance_layers.empty())
      //  layers.append_range(ext->instance_layers);
      
      u32 glfwExtensionCount = 0;
      const char** glfwExtensions = glfwGetRequiredInstanceExtensions(&glfwExtensionCount);
      std::vector<const char*> extensions(glfwExtensions, glfwExtensions + glfwExtensionCount);

      vk::InstanceCreateInfo createInfo({}, &appInfo, layers, extensions);
      vk_inst = vk::raii::Instance(ctx(), createInfo);
    }



    /// Select Device
    l_select_device: {
      const auto phys_devs = vk::raii::PhysicalDevices(vk_inst);

      i32 phys_idx{-1};

      for (int i = 0; i < phys_devs.size(); i++)
        if (f_is_suitable(phys_devs[i])) {
          phys_idx = i;
          break;
        }

      if (phys_idx == -1)
        throw rhi_error("no gpu was found to start the engine.");
      
      vk_phys_dev = phys_devs[phys_idx];
        
      if (!f_is_suitable(vk_phys_dev))
        throw rhi_error("the selected GPU is not supported by mochi.");

      auto dev_props = vk_phys_dev.getProperties();
      static const char* dev_type_names[] = { "Other", "Integrated", "Discrete", "Virtual", "CPU" };
      auto type_idx = static_cast<u32>(dev_props.deviceType);
      ME_LOG_VERB("gpu selected: {} ({})",
        dev_props.deviceName.data(),
        type_idx < 5 ? dev_type_names[type_idx] : "Unknown"
      )
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
      VK_EXT_DESCRIPTOR_HEAP_EXTENSION_NAME,
      VK_EXT_DESCRIPTOR_BUFFER_EXTENSION_NAME,
      VK_EXT_HOST_IMAGE_COPY_EXTENSION_NAME,
    };
    //if (ext && !ext->device_extensions.empty())
    //  extensions.append_range(ext->device_extensions);


    vk::PhysicalDeviceFeatures features{};
    features.samplerAnisotropy = VK_TRUE;
    features.fillModeNonSolid = VK_TRUE;
    features.shaderInt64 = VK_TRUE;


    vk::PhysicalDeviceVulkan12Features features12{};
    features12.bufferDeviceAddress = VK_TRUE;
    
    vk::PhysicalDeviceVulkan13Features features13{};
    features12.pNext = &features13;
    features13.dynamicRendering = VK_TRUE;
    features13.synchronization2 = VK_TRUE;

    vk::PhysicalDeviceVulkan14Features features14{};
    features13.pNext = &features14;
    features14.hostImageCopy = VK_TRUE;

    vk::PhysicalDeviceDescriptorHeapFeaturesEXT desc_heap{};
    desc_heap.descriptorHeap = VK_TRUE;
    features14.pNext = &desc_heap;

    vk::PhysicalDeviceDescriptorBufferFeaturesEXT desc_buffer{};
    desc_buffer.descriptorBuffer = VK_TRUE;
    desc_heap.pNext = &desc_buffer;


    vk::DeviceCreateInfo dev_info({}, queue_infos, {}, extensions, &features, &features12);
    vk_device = vk::raii::Device(vk_phys_dev, dev_info);


    bool main_q_set = false;
    for (u32 i{}; i < props.size(); i++) {
      auto flags = props[i].queueFlags;
      u32 count = props[i].queueCount;

      for (u32 q_idx{}; q_idx < count; q_idx++) {
        m_active_queues[i].push_back(vk::raii::Queue(vk_device, i, q_idx));

        // MAIN
        if (!main_q_set && flags & (vk::QueueFlagBits::eGraphics | vk::QueueFlagBits::eCompute)) {
          m_main_q = rhi::mng::Queue(i);
          main_q_set = true;
        }


        // GRAPHIC
        if (flags & vk::QueueFlagBits::eGraphics)
          graphics_q().add_primary(rhi::mng::Queue(i));
        

        // COMPUTE
        if (flags & vk::QueueFlagBits::eCompute) {
          if (!(flags & vk::QueueFlagBits::eGraphics))
            compute_q().add_primary(rhi::mng::Queue(i));
          else
            compute_q().add_secondary(rhi::mng::Queue(i));
        }


        // TRANSFER
        if (flags & vk::QueueFlagBits::eTransfer) {
          if (!(flags & vk::QueueFlagBits::eGraphics) && !(flags & vk::QueueFlagBits::eCompute))
            transfer_q().add_primary(rhi::mng::Queue(i));
          else
            transfer_q().add_secondary(rhi::mng::Queue(i));
        }
      }
    }


    vk::CommandPoolCreateInfo main_pool_info(vk::CommandPoolCreateFlagBits::eResetCommandBuffer, main_q().family());
    m_mainPool = vk::raii::CommandPool(vk_device, main_pool_info);
  }



  fun VK_DeviceManager::f_is_suitable(vk::raii::PhysicalDevice phys_dev) -> bool {
    auto featureChain = phys_dev.getFeatures2<
      vk::PhysicalDeviceFeatures2, 
      vk::PhysicalDeviceVulkan12Features, 
      vk::PhysicalDeviceVulkan13Features,
      vk::PhysicalDeviceVulkan14Features,
      vk::PhysicalDeviceDescriptorHeapFeaturesEXT,
      vk::PhysicalDeviceDescriptorBufferFeaturesEXT
    >();

    auto features = featureChain.get<vk::PhysicalDeviceFeatures2>();
    auto features12 = featureChain.get<vk::PhysicalDeviceVulkan12Features>();
    auto features13 = featureChain.get<vk::PhysicalDeviceVulkan13Features>();
    auto features14 = featureChain.get<vk::PhysicalDeviceVulkan14Features>();
    auto desc_heap = featureChain.get<vk::PhysicalDeviceDescriptorHeapFeaturesEXT>();
    auto desc_buffer = featureChain.get<vk::PhysicalDeviceDescriptorBufferFeaturesEXT>();


    return
      features.features.samplerAnisotropy &&
      features.features.fillModeNonSolid &&
      features.features.shaderInt64 &&
      features12.bufferDeviceAddress &&
      features13.dynamicRendering &&
      features13.synchronization2 &&
      features14.hostImageCopy &&
      desc_heap.descriptorHeap &&
      desc_buffer.descriptorBuffer;
  }


  fun VK_DeviceManager::getMainBuffer(u32 count) -> vk::raii::CommandBuffers {
    vk::CommandBufferAllocateInfo alloc_info(*m_mainPool, vk::CommandBufferLevel::ePrimary, count);
    return vk::raii::CommandBuffers(vk_device, alloc_info);
  }

  fun VK_DeviceManager::initDescriptorHeap(rhi::mng::AllocManager &alloc_mgr) -> void {
    // We assume Max 1024 textures for now
    u32 max_textures = 1024;
    
    // Query descriptor size for images/samplers
    vk::PhysicalDeviceDescriptorHeapPropertiesEXT heap_props;
    
    vk::PhysicalDeviceProperties2 props2 = {};
    props2.pNext = &heap_props;
    
    // Call the C API function
    vkGetPhysicalDeviceProperties2(*vk_phys_dev, props2);
    
    m_descriptor_size = heap_props.imageDescriptorSize;
    if (m_descriptor_size == 0) m_descriptor_size = 32; // Fallback estimate
    m_sampler_descriptor_size = heap_props.samplerDescriptorSize;
    if (m_sampler_descriptor_size == 0) m_sampler_descriptor_size = 32; // Fallback estimate

    m_descriptor_heap = alloc_mgr.allocBuffer(
      m_descriptor_size * max_textures,
      rhi::BufferUsage::DeviceAddress | rhi::BufferUsage::DescriptorHeap,
      rhi::mng::AllocationCreate::Mapped, // Host mapped
      rhi::mng::AllocationLocation::Auto
    );

    m_sampler_heap = alloc_mgr.allocBuffer(
      m_sampler_descriptor_size * max_textures,
      rhi::BufferUsage::DeviceAddress | rhi::BufferUsage::DescriptorHeap,
      rhi::mng::AllocationCreate::Mapped, // Host mapped
      rhi::mng::AllocationLocation::Auto
    );

    ME_LOG_VERB("descriptor heap initialized (capacity: {})", max_textures)
  }

  fun VK_DeviceManager::writeTextureDescriptor(sptr<rhi::ImageView2> view, sptr<rhi::Sampler2> sampler) -> u32 {
    u32 id = allocate_descriptor_id();
    u64 offset = id * m_descriptor_size;
    
    VkSampler sampler_handle = static_cast<vulkan::VK_Sampler2*>(sampler.get())->get();

    VkDescriptorImageInfo image_info = {};
    image_info.imageView   = static_cast<vulkan::VK_ImageView2*>(view.get())->get();
    image_info.imageLayout = VK_IMAGE_LAYOUT_SHADER_READ_ONLY_OPTIMAL;
    
    VkDescriptorGetInfoEXT get_info = {};
    get_info.sType = VK_STRUCTURE_TYPE_DESCRIPTOR_GET_INFO_EXT;
    get_info.type  = VK_DESCRIPTOR_TYPE_SAMPLED_IMAGE;
    get_info.data.pSampledImage = &image_info;


    vk_device.getDescriptorEXT(get_info, m_descriptor_size, (u8*)m_descriptor_heap->mapped() + offset);
    m_descriptor_heap->flush(offs(offset, m_descriptor_size));

    
    // Write sampler descriptor to sampler heap
    VkDescriptorGetInfoEXT sampler_get_info = {};
    sampler_get_info.sType = VK_STRUCTURE_TYPE_DESCRIPTOR_GET_INFO_EXT;
    sampler_get_info.type  = VK_DESCRIPTOR_TYPE_SAMPLER;
    sampler_get_info.data.pSampler = &sampler_handle;
    
    u64 sampler_offset = id * m_sampler_descriptor_size;
    vk_device.getDescriptorEXT(sampler_get_info, m_sampler_descriptor_size, (u8*)m_sampler_heap->mapped() + sampler_offset);
    m_sampler_heap->flush(offs(sampler_offset, m_sampler_descriptor_size));
    
    return id;
  }

}
