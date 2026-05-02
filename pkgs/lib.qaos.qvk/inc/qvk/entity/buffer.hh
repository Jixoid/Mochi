/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#pragma once

#include "Basis.hh"
#include "qvk/types.hh"
#include <vulkan/vulkan.hpp>
#include <vulkan/vulkan_raii.hpp>



namespace qvk
{

  template<>
  struct info<buffer>
  {
    private:
      explicit inline info<buffer>(u64 stride, std::vector<qvk::gt> items)
        : m_stride(stride)
        , m_items(items)
      {}

    public:
      static fun make(core &core, u64 stride, std::vector<qvk::gt> items) -> info<buffer>*;


    private:
      std::vector<qvk::gt> m_items;
      u64 m_stride;

    public:
      inline fun items() { return m_items; }
      inline fun stride() { return m_stride; }
  };




  struct buffer
  {
    friend struct memory;

    private:
      explicit buffer(device &device, memory &memory, info<buffer> *info, u64 count, vk::BufferUsageFlags usage, vk::MemoryPropertyFlags properties);

      ~buffer();


    private:
      info<buffer> *m_info{};
      vk::raii::Buffer m_buffer;
      vk::raii::DeviceMemory m_memory;
      vk::DeviceSize m_size;
      void* m_mapped{};

    public:
      inline fun  info() { return m_info; }
      inline fun& get() { return m_buffer; }
      inline fun& memory() const { return m_memory; }
      inline fun  size() const { return m_size; }
      inline fun  mapped() const { return m_mapped; }
  };

}
