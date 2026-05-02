/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#pragma once

#include "Basis.hh"
#include "qvk/entity/buffer.hh"
#include "qvk/module/device.hh"
#include "qvk/module/renderer.hh"
#include "qvk/types.hh"
#include <algorithm>
#include <cassert>
#include <functional>
#include <vulkan/vulkan.hpp>
#include <vulkan/vulkan_raii.hpp>



namespace qvk
{

  struct memory
  {
    public:
      explicit memory(qvk::device &device, qvk::renderer &renderer);


    private:
      qvk::device &m_device;
      qvk::renderer &m_renderer;
      bool m_sharedMemory{};

      u32 m_pushConstant;
      u64 m_uniformRange;
      u32 m_uniformAlign;
      u64 m_storageRange;
      u32 m_storageAlign;
      u64 m_allocCount;
      u64 m_mapAlign;
      u64 m_vramSize;


    public:
      inline fun sharedMemory() { return m_sharedMemory; }

      inline fun pushConstant() { return m_pushConstant; }
      inline fun uniformRange() { return m_uniformRange; }
      inline fun uniformAlign() { return m_uniformAlign; }
      inline fun storageRange() { return m_storageRange; }
      inline fun storageAlign() { return m_storageAlign; }


    private: // Ownership
      std::tuple<
        std::vector<buffer*>,
        std::vector<info<buffer>*>,

        std::vector<pipeline*>,
        std::vector<info<pipeline>*>,
        
        std::vector<object*>
      > m_owned;

    public:
      template <typename T>
      inline fun push(T *obj) {
        auto &vec = std::get<std::vector<T*>>(m_owned);

        assert(std::find(vec.begin(), vec.end(), obj) == vec.end() && "alread owned");

        vec.push_back(obj);
      }

      template <typename T>
      inline fun& list() { return std::get<std::vector<T*>>(m_owned); }
    

    public:
      fun find_memory_type(u32 type_filter, vk::MemoryPropertyFlags properties) -> u32;

      
    private:
      fun load_UMA_UniformBuffer(info<buffer> *info, u64 count, std::function<void (void*)> data) -> buffer*;
      fun load_UMA_StorageBuffer(info<buffer> *info, u64 count, std::function<void (void*)> data) -> buffer*;
      fun load_UMA_VertexBuffer(info<buffer> *info, u64 count, std::function<void (void*)> data) -> buffer*;

      fun load_DISC_UniformBuffer(info<buffer> *info, u64 count, std::function<void (void*)> data) -> buffer*;
      fun load_DISC_StorageBuffer(info<buffer> *info, u64 count, std::function<void (void*)> data) -> buffer*;
      fun load_DISC_VertexBuffer(info<buffer> *info, u64 count, std::function<void (void*)> data) -> buffer*;

    public:
      inline fun load_UniformBuffer(info<buffer> *info, u64 count, std::function<void (void*)> data) -> buffer* { return m_sharedMemory ? load_UMA_UniformBuffer(info, count, data) : load_DISC_UniformBuffer(info, count, data); }
      inline fun load_StorageBuffer(info<buffer> *info, u64 count, std::function<void (void*)> data) -> buffer* { return m_sharedMemory ? load_UMA_StorageBuffer(info, count, data) : load_DISC_StorageBuffer(info, count, data); }
      inline fun load_VertexBuffer(info<buffer> *info, u64 count, std::function<void (void*)> data) -> buffer* { return m_sharedMemory ? load_UMA_VertexBuffer(info, count, data) : load_DISC_VertexBuffer(info, count, data); }

  };

}
