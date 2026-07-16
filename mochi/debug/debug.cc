/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#include "mochi/basis.hh"
#include "mochi/debug/debug.hh"
#include <chrono>
#include <format>
#include <iostream>
#include <string_view>



namespace mochi::debug
{

  auto boot = std::chrono::steady_clock::now();



  fun debug(ModuleInfo &mod, MsgType typ, std::string_view str) -> void {
    
    auto now = std::chrono::steady_clock::now() - boot;
    auto ms = std::chrono::duration_cast<std::chrono::milliseconds>(now).count();

    std::cerr << std::format("{} [{}.{:03}] @{} {}\n", static_cast<char>(typ), ms/1000, ms%1000, mod.name(), str);
  }

}
