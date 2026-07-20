/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/

#include "mochi/systems/plugin_system.hh"
#include "mochi/basis.hh"
#include "mochi/debug/debug.hh"
#include "mochi/plugin/plugin.hh"
#include "mochi/types.hh"
#include <filesystem>

#ifdef __unix__
  #include <dlfcn.h>
#endif

namespace fs = std::filesystem;



namespace mochi::sys
{

  PluginSystem::PluginSystem(Engine &eng): m_eng(eng) {}
  
  PluginSystem::~PluginSystem() {
    while (!m_loadedPlugins.empty()) {
      unload_plugin(m_loadedPlugins.begin()->first);
    }
  }


  fun PluginSystem::hook(this PluginSystem& self, IPlugin* inst, HookThing thing, PluginFunc func) -> bool {
    if (!inst || !func) return false;

    self.m_hooks[static_cast<u64>(thing)].push_back({inst, func});
    
    return true;
  }

  fun PluginSystem::dispatch(HookThing thing) -> void {
    for (const auto &x: m_hooks[static_cast<u64>(thing)])
      x.second(x.first);
  }


  fun PluginSystem::load_all_plugin() -> void {
    auto path = fs::path("plugins");

    if (!fs::exists(path) || !fs::is_directory(path)) {
      ME_LOG_LOG("nothing was installed because the plugins/ directory was not found!");
      return;
    }

    for (const auto &entry: fs::directory_iterator(path))
      if (entry.is_regular_file()) {
        auto ext = entry.path().extension().string();
        if (
          #if defined (__unix__)
          ext == ".so"
          #elif defined (_WIN32)
          ext == ".dll"
          #elif defined (__Apple__)
          ext == ".dylib"
          #endif
        ){
          load_plugin(entry.path().string());
        }
      }
  }


  fun PluginSystem::load_plugin(std::string_view _path) -> bool {
    auto entry = fs::directory_entry(_path);
    auto name = entry.path().filename().string();

    if (m_loadedPlugins.contains(name)) return true;

    PluginContext ctx{};


    #ifdef __unix__
      ctx.lib_handle = dlopen(entry.path().c_str(), RTLD_NOW);
      if (!ctx.lib_handle) {
        ME_LOG_WARN("plugin \"{}\" loading failed: {}", name, dlerror())
        return false;
      }

      using sign = fun (*)() -> IPlugin*;
      sign NFunc = (sign)dlsym(ctx.lib_handle, "Mochi_NewPlugin");
      if (!NFunc) {
        ME_LOG_WARN("plugin \"{}\" missing entry point Mochi_NewPlugin", name)
        dlclose(ctx.lib_handle);
        return false;
      }

      ctx.instance = NFunc();
    #elif defined(_WIN32)
      // Windows'ta path::c_str() wchar_t* döndürür, LoadLibraryW ile tam uyumludur
      ctx.lib_handle = (void*)LoadLibraryW(entry.path().c_str());
      if (!ctx.lib_handle) {
        ME_LOG_WARN("plugin \"{}\" loading failed", name)
        return false;
      }

      using sign = fun (*)() -> IPlugin*;
      sign NFunc = (sign)GetProcAddress((HMODULE)ctx.lib_handle, "Mochi_NewPlugin");
      if (!NFunc) {
        ME_LOG_WARN("plugin \"{}\" missing entry point Mochi_NewPlugin", name)
        FreeLibrary((HMODULE)ctx.lib_handle);
        return false;
      }

      ctx.instance = NFunc();
    #endif

    if (!ctx.instance) {
      ME_LOG_WARN("plugin \"{}\" failed to instantiate instance", name)
      #ifdef __unix__
        dlclose(ctx.lib_handle);
      #elif defined(_WIN32)
        FreeLibrary((HMODULE)ctx.lib_handle);
      #endif
      return false;
    }


    auto inst = ctx.instance;


    HookFunc hook = [this, inst](HookThing thing, PluginFunc func) -> bool {
      return this->hook(inst, thing, func);
    };


    inst->on_load(m_eng, hook);

    m_loadedPlugins[name] = ctx;
    ME_LOG_VERB("plugin \"{}\" successfully loaded", name)
    return true;
  }

  fun PluginSystem::unload_plugin(std::string_view name) -> void {
    auto it = m_loadedPlugins.find(std::string(name));
    if (it == m_loadedPlugins.end()) return;

    auto& ctx = it->second;

    if (ctx.instance) {
      ctx.instance->on_unload();

      ctx.instance->free(); 
      ctx.instance = nil;
    }

    if (ctx.lib_handle) {
      #ifdef __unix__
        dlclose(ctx.lib_handle);
      #elif defined(_WIN32)
        FreeLibrary((HMODULE)ctx.lib_handle);
      #endif
      ctx.lib_handle = nil;
    }

    ME_LOG_VERB("plugin \"{}\" successfully unloaded", it->first)
    m_loadedPlugins.erase(it);
  }

}
