/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#pragma once

#include "Basis.hh"
#include <vulkan/vulkan_raii.hpp>
#include <vulkan/vulkan.h>



namespace qvk
{

  struct queue_group
  {
    friend struct device;
    
    private:
      std::vector<vk::raii::Queue> m_primary;   // Uzmanlaşmış
      std::vector<vk::raii::Queue> m_secondary; // Paylaşımlı

    public:
      inline fun available() const { return !m_primary.empty() || !m_secondary.empty(); }
    
      inline fun& best() const {
        return !m_primary.empty() ? m_primary.front() : m_secondary.front();
      }

      inline fun& bests() const {
        return !m_primary.empty() ? m_primary : m_secondary;
      }

  };


  struct device
  {
    public:
      explicit device(vk::raii::PhysicalDevice phys_dev);


    private:
      vk::raii::PhysicalDevice vk_phys_dev;
      vk::raii::Device vk_device;
      queue_group  m_graphics_q, m_compute_q, m_transfer_q;

      struct { u32 graphics, compute, transfer; } vk_indices;


    public:
      inline fun& phys_dev() { return vk_phys_dev; }
      inline fun& vdevice() { return vk_device; }
      inline fun& graphics_q() { return m_graphics_q; }
      inline fun& compute_q() { return m_compute_q; }
      inline fun& transfer_q() { return m_transfer_q; }
  };

}
