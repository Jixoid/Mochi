/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/

#pragma once

#include "mochi/basis.hh"
#include "mochi/plugin/plugin.hh"
#include "mochi/types.hh"
#include <unordered_map>



namespace mochi::sys
{

  struct PluginContext {
    void* lib_handle{nil};
    IPlugin* instance{nil};
  };


  struct PluginSystem: noncopy {
    public:
      explicit PluginSystem(Engine &eng);
      ~PluginSystem();

    private:
      Engine &m_eng;
      std::unordered_map<std::string, PluginContext> m_loadedPlugins;

      std::array<std::vector<std::pair<IPlugin*, PluginFunc>>, static_cast<u64>(HookThing::__HOOK_COUNT)> m_hooks;

    private:
      fun hook(this PluginSystem& self, IPlugin* inst, HookThing thing, PluginFunc func) -> bool;
    
    public:
      fun dispatch(HookThing thing) -> void;

      fun load_all_plugin() -> void;
      
      fun load_plugin(std::string_view path) -> bool;
      fun unload_plugin(std::string_view name) -> void;
  };

}
