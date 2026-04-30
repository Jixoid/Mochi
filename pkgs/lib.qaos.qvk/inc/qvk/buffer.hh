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

  struct BufferInfo
  {
    void *Data{};
    u0    Size{};
  };


  struct buffer
  {
    public:
      explicit buffer(qvk::memory &memory, qvk::device &device, vk::DeviceSize size, vk::BufferUsageFlags usage, vk::MemoryPropertyFlags properties);

      ~buffer();


    private:
      vk::raii::Buffer m_buffer;
      vk::raii::DeviceMemory m_memory;
      vk::DeviceSize m_size;
      void* m_mapped{};

    public:
      inline fun& get() { return m_buffer; }
      inline fun& memory() const { return m_memory; }
      inline fun  size() const { return m_size; }
      inline fun  mapped() const { return m_mapped; }
  };

}
