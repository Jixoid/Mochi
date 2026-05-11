/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#include "mochi/basis.hh"
#include "mochi/module/bridge.hh"
#include "mochi/module/device.hh"
#include "mochi/module/memory.hh"
#include "mochi/ecs/camera.hh"
#include "mochi/ecs/point_light.hh"
#include "mochi/rhi/buffer.hh"
#include "mochi/rhi/rhi.hh"
#include "mochi/types.hh"



namespace mochi::module
{

  memory::memory(bridge &bridge, device &device)
    : m_bridge(bridge)
    , m_device(device)
  {
    auto props = device.phys_dev().getProperties();
    vk::PhysicalDeviceMemoryProperties mem_props = device.phys_dev().getMemoryProperties();

    m_pushConstant = props.limits.maxPushConstantsSize;

    m_uniformRange = props.limits.maxUniformBufferRange;
    m_uniformAlign = props.limits.minUniformBufferOffsetAlignment;

    m_storageRange = props.limits.maxStorageBufferRange;
    m_storageAlign = props.limits.minStorageBufferOffsetAlignment;

    m_allocCount = props.limits.maxMemoryAllocationCount;
    m_mapAlign   = props.limits.minMemoryMapAlignment;

    m_vramSize = {};
    for (auto &X: mem_props.memoryHeaps)
      if (X.flags & vk::MemoryHeapFlagBits::eDeviceLocal)
        m_vramSize += X.size;


    u64 visible_vram{};
    auto target_flags = vk::MemoryPropertyFlagBits::eDeviceLocal | vk::MemoryPropertyFlagBits::eHostVisible | vk::MemoryPropertyFlagBits::eHostCoherent;

    for (auto &X: mem_props.memoryTypes)
      if ((X.propertyFlags & target_flags) == target_flags) {
        u32 heap_index = X.heapIndex;
        u64 heap_size = mem_props.memoryHeaps[heap_index].size;
        if (heap_size > visible_vram)
          visible_vram = heap_size;
      }

    if (props.deviceType == vk::PhysicalDeviceType::eIntegratedGpu)
      m_sharedMemory = true;
    else
      m_sharedMemory = (m_vramSize > 0 && visible_vram >= (m_vramSize * 0.9));



    VmaAllocatorCreateInfo allocatorInfo = {};
    allocatorInfo.physicalDevice = *device.phys_dev();
    allocatorInfo.device = *device.vdevice();
    allocatorInfo.instance = *bridge.inst();
    allocatorInfo.vulkanApiVersion = VK_API_VERSION_1_4;

    vmaCreateAllocator(&allocatorInfo, &m_allocator);


    vk::CommandPoolCreateInfo pool_info(
      vk::CommandPoolCreateFlagBits::eTransient,
      device.transfer_q().best().family
    );
    m_transfer_pool = vk::raii::CommandPool(device.vdevice(), pool_info);
  }

  memory::~memory()
  {
    m_camera_ubo.reset();
    m_light_ubo.reset();
    vmaDestroyAllocator(m_allocator);
  }

  


  fun memory::camera_ubo(u64 required_count) -> sptr<rhi::buffer>
  {
    if (!m_camera_ubo || (m_camera_ubo->size() / ecs::camera_i->stride()) < required_count) {
      m_camera_ubo = load_UniformBuffer(ecs::camera_i, std::max<u64>(10, required_count), [](void*){});
    }
    return m_camera_ubo;
  }

  fun memory::light_ubo(u64 required_count) -> sptr<rhi::buffer>
  {
    if (!m_light_ubo || (m_light_ubo->size() / ecs::point_light_i->stride()) < required_count) {
      m_light_ubo = load_UniformBuffer(ecs::point_light_i, std::max<u64>(100, required_count), [](void*){});
    }
    return m_light_ubo;
  }



  fun memory::load_UMA_UniformBuffer(sptr<rhi::info<rhi::buffer>> info, u64 count, std::function<void (void*)> data) -> sptr<rhi::buffer>
  {
    auto ret = rhi::buffer::make(
      m_device, *this, info, count,
      rhi::BufferUsage::UniformBuffer,
      flags(rhi::BufferCreate::Mapped) | rhi::BufferCreate::HostSequentialWrite
    );

    if (data && ret->mapped()) data(ret->mapped());

    return ret;
  }

  fun memory::load_DISC_UniformBuffer(sptr<rhi::info<rhi::buffer>> info, u64 count, std::function<void (void*)> data) -> sptr<rhi::buffer>
  {
    auto ret = rhi::buffer::make(
      m_device, *this, info, count, 
      rhi::BufferUsage::UniformBuffer,
      flags(rhi::BufferCreate::Mapped) | rhi::BufferCreate::HostSequentialWrite
    );

    if (data && ret->mapped()) data(ret->mapped());

    return ret;
  }


  
  fun memory::load_UMA_StorageBuffer(sptr<rhi::info<rhi::buffer>> info, u64 count, std::function<void (void*)> data) -> sptr<rhi::buffer>
  {
    auto ret = rhi::buffer::make(
      m_device, *this, info, count,
      rhi::BufferUsage::StorageBuffer,
      flags(rhi::BufferCreate::Mapped) | rhi::BufferCreate::HostSequentialWrite
    );

    if (data && ret->mapped()) data(ret->mapped());

    return ret;
  }

  fun memory::load_DISC_StorageBuffer(sptr<rhi::info<rhi::buffer>> info, u64 count, std::function<void (void*)> data) -> sptr<rhi::buffer>
  {
    auto ret = rhi::buffer::make(
      m_device, *this, info, count,
      flags(rhi::BufferUsage::StorageBuffer) | rhi::BufferUsage::TransferDst,
      flags(rhi::BufferCreate::Mapped) | rhi::BufferCreate::HostSequentialWrite
    );

    if (data) {
      auto host_buffer = rhi::buffer::make(
        m_device, *this, info, count,
        rhi::BufferUsage::TransferSrc,
        flags(rhi::BufferCreate::Mapped) | rhi::BufferCreate::HostSequentialWrite
      );
      data(host_buffer->mapped());


      vk::CommandBufferAllocateInfo cmd_alloc_info(*m_transfer_pool, vk::CommandBufferLevel::ePrimary, 1);
      vk::raii::CommandBuffer temp_cmd = std::move(vk::raii::CommandBuffers(m_device.vdevice(), cmd_alloc_info).front());
      

      vk::BufferCopy copy_region(0, 0, ret->size());

      temp_cmd.begin({vk::CommandBufferUsageFlagBits::eOneTimeSubmit});
      temp_cmd.copyBuffer(host_buffer->get(), ret->get(), copy_region);
      temp_cmd.end();

      vk::SubmitInfo submit_info({}, {}, *temp_cmd, {});
      m_device.transfer_q().best().queue.submit(submit_info, nil);
      m_device.transfer_q().best().queue.waitIdle();
    }
    
    return ret;
  }

  

  fun memory::load_UMA_VertexBuffer(sptr<rhi::info<rhi::buffer>> info, u64 count, std::function<void (void*)> data) -> sptr<rhi::buffer>
  {
    auto ret = rhi::buffer::make(
      m_device, *this, info, count,
      rhi::BufferUsage::VertexBuffer,
      flags(rhi::BufferCreate::Mapped) | rhi::BufferCreate::HostSequentialWrite
    );

    if (data && ret->mapped()) data(ret->mapped());

    return ret;
  }

  fun memory::load_DISC_VertexBuffer(sptr<rhi::info<rhi::buffer>> info, u64 count, std::function<void (void*)> data) -> sptr<rhi::buffer>
  {
    auto ret = rhi::buffer::make(
      m_device, *this, info, count,
      flags(rhi::BufferUsage::VertexBuffer) | rhi::BufferUsage::TransferDst,
      flags(rhi::BufferCreate::Mapped) | rhi::BufferCreate::HostSequentialWrite
    );

    if (data) {
      auto host_buffer = rhi::buffer::make(
        m_device, *this, info, count, 
        rhi::BufferUsage::TransferSrc,
        flags(rhi::BufferCreate::Mapped) | rhi::BufferCreate::HostSequentialWrite
      );
      data(host_buffer->mapped());


      vk::CommandBufferAllocateInfo cmd_alloc_info(*m_transfer_pool, vk::CommandBufferLevel::ePrimary, 1);
      vk::raii::CommandBuffer temp_cmd = std::move(vk::raii::CommandBuffers(m_device.vdevice(), cmd_alloc_info).front());
      vk::BufferCopy copy_region(0, 0, ret->size());
      vk::MemoryBarrier memory_barrier(
        vk::AccessFlagBits::eTransferWrite,
        vk::AccessFlagBits::eVertexAttributeRead
      );

      temp_cmd.begin({vk::CommandBufferUsageFlagBits::eOneTimeSubmit});
      temp_cmd.copyBuffer(host_buffer->get(), ret->get(), copy_region);

      temp_cmd.pipelineBarrier(
        vk::PipelineStageFlagBits::eTransfer,
        vk::PipelineStageFlagBits::eVertexInput,
        {},
        {memory_barrier},
        {},
        {}
      );
      
      temp_cmd.end();

      vk::SubmitInfo submit_info({}, {}, *temp_cmd, {});
      m_device.transfer_q().best().queue.submit(submit_info, nil);
      m_device.transfer_q().best().queue.waitIdle();
    }
    
    return ret;
  }

}
