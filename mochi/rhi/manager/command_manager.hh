/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/

#pragma once

#include "mochi/basis.hh"
#include "mochi/rhi/manager/device_manager.hh"
#include "mochi/rhi/command.hh"
#include "mochi/types.hh"



namespace mochi::rhi
{
  // External
  extern "C" fun MochiRHI_MakeCommandManager(rhi::DeviceManager &dmng) -> CommandManager*;


  // Interface
  struct CommandManager: noncopy {
    protected:
      CommandManager(rhi::DeviceManager &dmng): m_dmng(dmng) {}

    public:
      virtual ~CommandManager() = default;

      static fun make(rhi::DeviceManager &device) {
        return make_sptr(MochiRHI_MakeCommandManager(device));
      }

    protected:
      rhi::DeviceManager &m_dmng;

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
