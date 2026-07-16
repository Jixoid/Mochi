/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#pragma once

#include "mochi/basis.hh"
#include <string_view>



namespace mochi::debug
{

  struct ModuleInfo {
    public:
      ModuleInfo(const char* name)
        : m_name(name)
      {}

      ~ModuleInfo() {
        m_name = std::string_view{};
      }

    private:
      std::string_view m_name;

    public:
      fun name() { return m_name; }
  };

  enum struct MsgType: u8 { Hint = 0x48, Warning = 0x57, Error = 0x45, Fatal = 0x46 };


  fun debug(ModuleInfo &mod, MsgType typ, std::string_view str) -> void;

}
