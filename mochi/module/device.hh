/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#pragma once

#include "mochi/basis.hh"
#include "mochi/rhi/buffer.hh"
#include <vulkan/vulkan_raii.hpp>
#include <vulkan/vulkan.h>



namespace mochi::module
{

  struct queue
  {
    public:
      queue(nil_t): m_queue(nil), m_family(-1) {}

      queue(vk::raii::Queue queue, u32 family): m_queue(queue), m_family(family) {}

    private:
      vk::raii::Queue m_queue;
      u32 m_family;

    public:
      inline fun get() const { return m_queue; }
      inline fun family() const { return m_family; }
  };


  /** @brief Represents a group of Vulkan queues for a specific capability. */
  struct queue_group
  {
    friend struct device;
    
    private:
      std::vector<queue> m_primary;   // Dedicated (specialized) queues
      std::vector<queue> m_secondary; // Shared queues

    public:
      /** @brief Check if any queue is available in this group. */
      inline fun available() const { return !m_primary.empty() || !m_secondary.empty(); }
    
      /** @brief Get the best available single queue (prefers dedicated). */
      inline fun& best() const {
        return !m_primary.empty() ? m_primary.front() : m_secondary.front();
      }

      /** @brief Get the best available list of queues (prefers dedicated). */
      inline fun& bests() const {
        return !m_primary.empty() ? m_primary : m_secondary;
      }

  };


  /** @brief Represents a Vulkan logical device and its associated resources. */
  struct device
  {
    public:
      /**
       * @brief Initialize a logical device from a physical device.
       * @param phys_dev The Vulkan physical device to create the logical device from.
       */
      explicit device(vk::raii::PhysicalDevice phys_dev);


    private:
      vk::raii::PhysicalDevice vk_phys_dev;
      vk::raii::Device vk_device;

      queue m_main_q;
      queue_group m_graphics_q, m_compute_q, m_transfer_q;

      vk::raii::CommandPool m_mainPool; // Has Graphic & Compute
      vk::raii::CommandPool m_transferPool;

      vk::raii::CommandBuffer m_transferBuf;
      std::vector<sptr<rhi::buffer>> m_transferBuf_refs;
      bool m_transferBuf_used{};


    public:
      inline fun& phys_dev() { return vk_phys_dev; }
      inline fun& get() { return vk_device; }
      inline fun& vdevice() { return vk_device; }
      
      inline fun& main_q() { return m_main_q; }
      inline fun& graphics_q() { return m_graphics_q; }
      inline fun& compute_q() { return m_compute_q; }
      inline fun& transfer_q() { return m_transfer_q; }

      inline fun& transferBuf() {
        if (!m_transferBuf_used) {
          m_transferBuf.reset();
          m_transferBuf.begin({vk::CommandBufferUsageFlagBits::eOneTimeSubmit});
          m_transferBuf_used = true;
        }
        return m_transferBuf;
      }

      inline fun addTransferBufRef(sptr<rhi::buffer> ref) { m_transferBuf_refs.push_back(ref); }

    public:
      fun flushTransferBuf() -> void;
      fun getMainBuffer(u32 count) -> vk::raii::CommandBuffers;
      fun getTransferBuffer(u32 count) -> vk::raii::CommandBuffers;
  };

}
