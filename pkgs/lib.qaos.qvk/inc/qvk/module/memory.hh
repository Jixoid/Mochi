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
#include "qvk/entity/visual.hh"
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


    #pragma region Limits
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
    #pragma endregion


    
    #pragma region OwnerShip
    private:
      std::tuple<
        std::vector<node*>,
        std::vector<camera*>,
        std::vector<light*>,
        std::vector<mesh*>,
        std::vector<visual*>,
      
        std::vector<buffer*>,

        std::vector<pipeline*>
        
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



    private: // Camera
      u64 m_camera_count{};
      qvk::buffer *m_camera_ubo{};
      fun prepare_camera(u32 count) -> void;
      
    public:
      fun find_camera(camera *cam) -> u64;
      fun sync_camera(camera *cam) -> void;
      inline fun camera_ubo() { return m_camera_ubo; }

      template <typename T>
        requires std::is_same_v<camera, T>
      inline fun push(T *obj) {
        auto &vec = std::get<std::vector<T*>>(m_owned);
        assert(std::find(vec.begin(), vec.end(), obj) == vec.end() && "alread owned");
        vec.push_back(obj);

        prepare_camera(++m_camera_count);
        sync_camera(obj);
      }

      template <typename T>
        requires std::is_same_v<camera, T>
      inline fun pop(T *obj) {
        auto &vec = std::get<std::vector<T*>>(m_owned);
        auto it = std::find(vec.begin(), vec.end(), obj);
        assert(it != vec.end() && "not registered");
        vec.erase(it);

        prepare_camera(--m_camera_count);
      }


    private: // Light
      u64 m_light_count{};
      qvk::buffer *m_light_ubo{};
      fun prepare_light(u32 count) -> void;
      
    public:
      fun find_light(light *cam) -> u64;
      fun sync_light(light *cam) -> void;
      inline fun light_ubo() { return m_light_ubo; }

      template <typename T>
        requires std::is_same_v<light, T>
      inline fun push(T *obj) {
        auto &vec = std::get<std::vector<T*>>(m_owned);
        assert(std::find(vec.begin(), vec.end(), obj) == vec.end() && "alread owned");
        vec.push_back(obj);

        prepare_light(++m_light_count);
        sync_light(obj);
      }

      template <typename T>
        requires std::is_same_v<light, T>
      inline fun pop(T *obj) {
        auto &vec = std::get<std::vector<T*>>(m_owned);
        auto it = std::find(vec.begin(), vec.end(), obj);
        assert(it != vec.end() && "not registered");
        vec.erase(it);

        prepare_light(--m_light_count);
      }
    #pragma endregion



    #pragma region Allocation
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
    #pragma endregion

  };

}
