/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/

#include "mochi/basis.hh"
#include "mochi/debug/debug.hh"


namespace mochi::rhi::vulkan {debug::ModuleInfo debug_module("DRV.VK", debug::LogType::Verbose);}
namespace mochi::rhi::vulkan::mng {debug::ModuleInfo debug_module("DRV.VK.MNG", debug::LogType::Verbose);}


namespace mochi::rhi::vulkan
{
  extern "C" {
    const char* MochiRHI_DriverInfo = "Vulkan";
    const u32   MochiRHI_DriverVers[4] = {0,0,0,0};
  }
  
}
