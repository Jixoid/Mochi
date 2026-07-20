/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/

#pragma once

#include "mochi/basis.hh"
#include "mochi/rhi/manager/device.hh"
#include "mochi/rhi/render_target.hh"
#include "mochi/types.hh"



namespace mochi::rhi
{
  // External
  extern "C" fun MochiRHI_MakeSwapchainManager(rhi::Device &dmng) -> SwapchainManager*;


  // Interface
  struct SwapchainManager: noncopy {
    protected:
      SwapchainManager(rhi::Device &dmng): m_dmng(dmng) {}

    public:
      virtual ~SwapchainManager() = default;

      static fun make(rhi::Device &device) {
        return make_sptr(MochiRHI_MakeSwapchainManager(device));
      }

    protected:
      rhi::Device &m_dmng;

    public:
      // Platforma özel pencere handle'ı ile swapchain ilklendirme (GLFW/SDL pencere işaretçisi)
      virtual fun init(void* windowHandle, u32 width, u32 height) -> void = 0;
      virtual fun resize(u32 width, u32 height) -> void = 0;

      // Swapchain akışı
      virtual fun acquireNextImage(void* signalSemaphore) -> u32 = 0;
      virtual fun present(u32 imageIndex, void* waitSemaphore) -> void = 0;

      // Çizim hedefi (Render Target) erişimi
      virtual fun getRenderTarget(u32 index) -> RenderTarget& = 0;
      virtual fun getRenderTargetCount() const -> u32 = 0;

      // Senkronizasyon
      virtual fun getRenderFinishedSemaphore(u32 image_index) -> void* = 0;
  };

}
