/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/

#pragma once

#include "mochi/basis.hh"
#include "mochi/types.hh"



namespace mochi::rhi
{
  // External
  extern "C" fun MochiRHI_MakeCommandManager(rhi::Device &device) -> rhi::CommandManager*;


  // Interface
  struct CommandManager: noncopy {
    protected:
      CommandManager(rhi::Device &device): m_device(device) {}

    public:
      virtual ~CommandManager() = default;

      static fun make(rhi::Device &device) {
        return make_uptr(MochiRHI_MakeCommandManager(device));
      }

    protected:
      rhi::Device &m_device;

    public:
      // Kareye ve iş parçacığına göre otomatik yönetilen komut tamponu tahsisi
      virtual fun allocateGraphicsCommand() -> sptr<Command> = 0;
      virtual fun allocateComputeCommand() -> sptr<Command> = 0;
      virtual fun allocateTransferCommand() -> sptr<Command> = 0;

      // Komut tamponlarını güvenli bir şekilde ilgili kuyruğa gönderme
      virtual fun submit(Command* cmd) -> void = 0;
      virtual fun submitWithSync(Command* cmd, void* waitSem, void* sigSem, void* fence) -> void = 0;
  };

}
