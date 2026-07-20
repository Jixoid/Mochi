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
#include <functional>



namespace mochi
{
  struct IPlugin;


  enum struct HookThing: u64 {
    Update = 0,

    __HOOK_COUNT = 1,
  };

  using PluginFunc = fun (*)(IPlugin*) -> void;

  using HookFunc = std::function<fun (HookThing, PluginFunc) -> bool>;



  struct IPlugin: noncopy {
    public:
      virtual ~IPlugin() = default;

    public:
      virtual fun free() -> void = 0;

      virtual fun on_load(Engine &eng, HookFunc hook) -> void = 0;
      virtual fun on_unload() -> void = 0;
  };

}
