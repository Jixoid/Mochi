/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/

#pragma once

#include "mochi/basis.hh"
#include "mochi/rhi/image.hh"
#include "mochi/rhi/sampler.hh"
#include "mochi/rhi/buffer.hh"
#include "mochi/types.hh"
#include <functional>



namespace mochi::rhi
{

  // Helpers
  struct Queue {
    public:
      explicit Queue() {}
      explicit Queue(u32 family): m_family(family) {}

    private:
      u32 m_family{};
      //vk::raii::Queue m_queue{nil};

    public:
      fun family() const { return m_family; }
  };


  struct QueueGroup: noncopy {
    friend struct DeviceManager;
    
    private:
      std::vector<Queue> m_primary;   // Dedicated (specialized) queues
      std::vector<Queue> m_secondary; // Shared queues

    public:
      fun add_primary(Queue q) -> void { m_primary.push_back(std::move(q)); }
      fun add_secondary(Queue q) -> void { m_secondary.push_back(std::move(q)); }

      fun available() const { return !m_primary.empty() || !m_secondary.empty(); }
    
      fun& best() const { return !m_primary.empty() ? m_primary.front() : m_secondary.front(); }
      fun& bests() const { return !m_primary.empty() ? m_primary : m_secondary; }
  };


  // External
  extern "C" fun MochiRHI_MakeDeviceManager(std::string_view appName, std::array<u16, 4> appVer) -> DeviceManager*;


  // Interface
  struct DeviceManager: noncopy {
    protected:
      DeviceManager() = default;

    public:
      virtual ~DeviceManager() = default;
      
      static fun make(std::string_view appName, std::array<u16, 4> appVer, std::function<i32 ()> GpuPicker) {
        return make_uptr(MochiRHI_MakeDeviceManager(appName, appVer));
      }

      virtual fun waitIdle() -> void = 0;

    protected:
      Queue m_main_q;
      QueueGroup m_graphics_q, m_compute_q, m_transfer_q;

    public:
      fun& main_q() { return m_main_q; }
      fun& graphics_q() { return m_graphics_q; }
      fun& compute_q() { return m_compute_q; }
      fun& transfer_q() { return m_transfer_q; }
      
      // Descriptor Management
      virtual fun initDescriptorHeap(rhi::AllocManager &alloc_mgr) -> void = 0;
      virtual fun writeTextureDescriptor(sptr<rhi::ImageView2> view, sptr<rhi::Sampler2> sampler) -> u32 = 0;
      virtual fun descriptor_heap() -> sptr<rhi::Buffer> = 0;
      virtual fun sampler_heap() -> sptr<rhi::Buffer> = 0;
  };

}
