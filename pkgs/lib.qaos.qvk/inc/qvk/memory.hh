/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#pragma once

#include "Basis.hh"
#include "qvk/buffer.hh"
#include "qvk/device.hh"
#include "qvk/renderer.hh"
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
      u64 m_uniformSize;
      u32 m_uniformAlign;
      u64 m_storageSize;
      u32 m_storageAlign;
      u64 m_allocCount;
      u64 m_mapAlign;
      u64 m_vramSize;


    public:
      inline fun sharedMemory() { return m_sharedMemory; }

      inline fun pushConstant() { return m_pushConstant; }
      inline fun uniformSize() { return m_uniformSize; }
      inline fun uniformAlign() { return m_uniformAlign; }
      inline fun storageSize() { return m_storageSize; }
      inline fun storageAlign() { return m_storageAlign; }


    private: // Ownership
      std::tuple<
        std::vector<camera*>,
        std::vector<buffer*>,
        std::vector<pipeline*>,
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
      fun load_UMA_UniformBuffer(u64 size, std::function<void (void*)> data) -> buffer*;
      fun load_UMA_StorageBuffer(u64 size, std::function<void (void*)> data) -> buffer*;
      fun load_UMA_VertexBuffer(u64 size, std::function<void (void*)> data) -> buffer*;

      fun load_DISC_UniformBuffer(u64 size, std::function<void (void*)> data) -> buffer*;
      fun load_DISC_StorageBuffer(u64 size, std::function<void (void*)> data) -> buffer*;
      fun load_DISC_VertexBuffer(u64 size, std::function<void (void*)> data) -> buffer*;

    public:
      inline fun load_UniformBuffer(u64 size, std::function<void (void*)> data) -> buffer* { return m_sharedMemory ? load_UMA_UniformBuffer(size, data) : load_DISC_UniformBuffer(size, data); }
      inline fun load_StorageBuffer(u64 size, std::function<void (void*)> data) -> buffer* { return m_sharedMemory ? load_UMA_StorageBuffer(size, data) : load_DISC_StorageBuffer(size, data); }
      inline fun load_VertexBuffer(u64 size, std::function<void (void*)> data) -> buffer* { return m_sharedMemory ? load_UMA_VertexBuffer(size, data) : load_DISC_VertexBuffer(size, data); }

  };

}
