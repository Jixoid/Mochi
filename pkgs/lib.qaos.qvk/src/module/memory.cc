/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#include "Basis.hh"
#include "qvk/module/renderer.hh"
#include "qvk/module/device.hh"
#include "qvk/module/memory.hh"
#include "qvk/entity/buffer.hh"
#include "qvk/types.hh"
#include <vulkan/vulkan.hpp>
#include <vulkan/vulkan_raii.hpp>
#include <cstring> // memcpy için eklendi


namespace qvk
{

  memory::memory(qvk::device &device, qvk::renderer &renderer)
    : m_device(device)
    , m_renderer(renderer)
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
  }



  fun memory::find_memory_type(u32 type_filter, vk::MemoryPropertyFlags properties) -> u32
  {
    auto mem_props = m_device.phys_dev().getMemoryProperties();

    for (u32 i{}; i < mem_props.memoryTypeCount; i++)
      if ((type_filter & (1 << i)) && (mem_props.memoryTypes[i].propertyFlags & properties) == properties) {
        return i;
      }
    
    throw std::runtime_error("Uygun GPU bellek türü bulunamadı!");
  }



  fun memory::load_UMA_UniformBuffer(info<buffer> *info, u64 count, std::function<void (void*)> data) -> buffer*
  {
    auto ret = new buffer(
      m_device, *this, info, count, 
      vk::BufferUsageFlagBits::eUniformBuffer,
      vk::MemoryPropertyFlagBits::eDeviceLocal | vk::MemoryPropertyFlagBits::eHostVisible | vk::MemoryPropertyFlagBits::eHostCoherent
    );

    if (data) data(ret->mapped());

    std::get<std::vector<buffer*>>(m_owned).push_back(ret);
    return ret;
  }

  fun memory::load_DISC_UniformBuffer(info<buffer> *info, u64 count, std::function<void (void*)> data) -> buffer*
  {
    auto ret = new buffer(
      m_device, *this, info, count, 
      vk::BufferUsageFlagBits::eUniformBuffer,
      vk::MemoryPropertyFlagBits::eHostVisible | vk::MemoryPropertyFlagBits::eHostCoherent
    );

    if (data) data(ret->mapped());

    std::get<std::vector<buffer*>>(m_owned).push_back(ret);
    return ret;
  }
  


  fun memory::load_UMA_StorageBuffer(info<buffer> *info, u64 count, std::function<void (void*)> data) -> buffer*
  {
    auto ret = new buffer(
      m_device, *this, info, count,
      vk::BufferUsageFlagBits::eStorageBuffer,
      vk::MemoryPropertyFlagBits::eDeviceLocal | vk::MemoryPropertyFlagBits::eHostVisible | vk::MemoryPropertyFlagBits::eHostCoherent
    );

    if (data) data(ret->mapped());

    std::get<std::vector<buffer*>>(m_owned).push_back(ret);
    return ret;
  }

  fun memory::load_DISC_StorageBuffer(info<buffer> *info, u64 count, std::function<void (void*)> data) -> buffer*
  {
    auto ret = new buffer(
      m_device, *this, info, count,
      vk::BufferUsageFlagBits::eStorageBuffer | vk::BufferUsageFlagBits::eTransferDst, 
      vk::MemoryPropertyFlagBits::eDeviceLocal
    );


    if (data) {
      // Host Buffer
      auto host_buffer = buffer(
        m_device, *this, info, count,
        vk::BufferUsageFlagBits::eTransferSrc, 
        vk::MemoryPropertyFlagBits::eHostVisible | vk::MemoryPropertyFlagBits::eHostCoherent
      );
      data(host_buffer.mapped());


      // Transfer
      vk::CommandBufferAllocateInfo alloc_info(*m_renderer.cmd_pool(), vk::CommandBufferLevel::ePrimary, 1);
      vk::raii::CommandBuffer temp_cmd = std::move(vk::raii::CommandBuffers(m_device.vdevice(), alloc_info).front());
      vk::BufferCopy copy_region(0, 0, ret->size());

      temp_cmd.begin({vk::CommandBufferUsageFlagBits::eOneTimeSubmit});
      temp_cmd.copyBuffer(*host_buffer.get(), ret->get(), copy_region);
      temp_cmd.end();

      vk::SubmitInfo submit_info({}, {}, *temp_cmd, {});
      m_device.transfer_q().best().submit(submit_info, nullptr);
      m_device.transfer_q().best().waitIdle();
    }
    
    std::get<std::vector<buffer*>>(m_owned).push_back(ret);
    return ret;
  }



  fun memory::load_UMA_VertexBuffer(info<buffer> *info, u64 count, std::function<void (void*)> data) -> buffer*
  {
    auto ret = new buffer(
      m_device, *this, info, count,
      vk::BufferUsageFlagBits::eVertexBuffer,
      vk::MemoryPropertyFlagBits::eDeviceLocal | vk::MemoryPropertyFlagBits::eHostVisible | vk::MemoryPropertyFlagBits::eHostCoherent
    );

    if (data) data(ret->mapped());

    std::get<std::vector<buffer*>>(m_owned).push_back(ret);
    return ret;
  }

  fun memory::load_DISC_VertexBuffer(info<buffer> *info, u64 count, std::function<void (void*)> data) -> buffer*
  {
    auto ret = new buffer(
      m_device, *this, info, count,
      vk::BufferUsageFlagBits::eVertexBuffer | vk::BufferUsageFlagBits::eTransferDst, 
      vk::MemoryPropertyFlagBits::eDeviceLocal
    );


    if (data) {
      // Host Buffer
      auto host_buffer = buffer(
        m_device, *this, info, count,
        vk::BufferUsageFlagBits::eTransferSrc, 
        vk::MemoryPropertyFlagBits::eHostVisible | vk::MemoryPropertyFlagBits::eHostCoherent
      );
      data(host_buffer.mapped());


      // Transfer
      vk::CommandBufferAllocateInfo alloc_info(*m_renderer.cmd_pool(), vk::CommandBufferLevel::ePrimary, 1);
      vk::raii::CommandBuffer temp_cmd = std::move(vk::raii::CommandBuffers(m_device.vdevice(), alloc_info).front());
      vk::BufferCopy copy_region(0, 0, ret->size());
      vk::MemoryBarrier memory_barrier(
        vk::AccessFlagBits::eTransferWrite,
        vk::AccessFlagBits::eVertexAttributeRead
      );

      temp_cmd.begin({vk::CommandBufferUsageFlagBits::eOneTimeSubmit});
      temp_cmd.copyBuffer(*host_buffer.get(), ret->get(), copy_region);

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
      m_device.transfer_q().best().submit(submit_info, nullptr);
      m_device.transfer_q().best().waitIdle();
    }
    
    std::get<std::vector<buffer*>>(m_owned).push_back(ret);
    return ret;
  }

  // *** Yeni Eklenen Update Fonksiyonunun Gövdesi ***
  void memory::update_buffer(qvk::buffer* target_buffer, const void* data, size_t size)
  {
      if (!target_buffer || !data) return;

      // Senin buffer sınıfında mapped() fonksiyonu zaten bellek adresini (void*) dönüyor
      void* mapped_memory = target_buffer->mapped(); 

      if (mapped_memory) {
          // Gelen yeni veriyi (Kamera Matrisini) GPU'nun map'lenmiş belleğine kopyala
          memcpy(mapped_memory, data, size);
      }
  }

}