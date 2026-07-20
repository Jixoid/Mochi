/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/

#pragma once

#include "mochi/basis.hh"
#include "mochi/rhi/manager/command_manager.hh"
#include "drivers/vulkan/manager/VKdevice.hh"
#include "drivers/vulkan/VKcommand.hh"
#include <vulkan/vulkan_raii.hpp>


namespace mochi::rhi::vulkan
{
  struct VK_CommandManager final : public rhi::CommandManager {
    public:
      explicit VK_CommandManager(rhi::Device &dmng);
      ~VK_CommandManager() override = default;

    public:
      fun allocateGraphicsCommand() -> sptr<Command> override;
      fun allocateComputeCommand() -> sptr<Command> override;
      fun allocateTransferCommand() -> sptr<Command> override;

      fun submit(Command* cmd) -> void override;
      fun submitWithSync(Command* cmd, void* waitSem, void* sigSem, void* fence) -> void override;
  };
}
