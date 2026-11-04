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
#include <stdexcept>
#include <string_view>
#include <filesystem>

namespace fs = std::filesystem;



namespace mochi::debug
{

  auto boot = std::chrono::steady_clock::now();



  fun log_str(ModuleInfo &mod, LogType typ, const std::source_location location, std::string_view str) -> std::string {

    struct writeHelp {
      char Sym;
      const char *Col;
    }
    static WriteHelp[] = {
      {'V', "\033[1;30m"},
      {'L', "\033[1;30m"},
      {'D', "\033[1;37m"},
      {'W', "\033[1;33m"},
      {'E', "\033[1;31m"},
      {'P', "\033[1;31m"},
    };

    const writeHelp &Help = WriteHelp[static_cast<u8>(typ)];

    
    auto now = std::chrono::steady_clock::now() - boot;
    auto ms = std::chrono::duration_cast<std::chrono::milliseconds>(now).count();

    if (static_cast<u8>(typ) >= static_cast<u8>(LogType::Warning))
      return std::format(
      "{}{} [{}.{:03}] @{} {}\n"
      "├─> ""\033[1;30m""func: {}\n{}"
      "╰─> ""\033[1;30m""file: {}:{}:{}"
      "\033[0m",
        Help.Col, Help.Sym,
        ms/1000, ms%1000,
        mod.name(), str,

        location.function_name(), Help.Col,

        (std::string)fs::path(location.file_name()).filename(),
        location.line(), location.column()
      );
    else
      return std::format("{}{} [{}.{:03}] @{} {}""\033[0m",
        Help.Col, Help.Sym,
        ms/1000, ms%1000,
        mod.name(),
        str
      );
  }

  fun log(ModuleInfo &mod, LogType typ, const std::source_location location, std::string_view str) -> void {
    std::cerr << log_str(mod, typ, location, str) << '\n';
  }

  [[gnu::noreturn]] fun log_nr(ModuleInfo &mod, const std::source_location location, std::string_view str) -> void {
    auto err = log_str(mod, LogType::Panic, location, str);

    std::cerr << err << std::endl;

    throw std::runtime_error(err);
  }

}


namespace mochi {
  namespace rhi     {debug::ModuleInfo debug_module{"ME.RHI",     debug::LogType::Verbose};}
  namespace ahi     {debug::ModuleInfo debug_module{"ME.AHI",     debug::LogType::Verbose};}
  namespace sys     {debug::ModuleInfo debug_module{"ME.SYS",     debug::LogType::Verbose};}
  namespace ecs     {debug::ModuleInfo debug_module{"ME.ECS",     debug::LogType::Verbose};}
  namespace math    {debug::ModuleInfo debug_module{"ME.MATH",    debug::LogType::Verbose};}
  namespace asset   {debug::ModuleInfo debug_module{"ME.ASSET",   debug::LogType::Verbose};}
  namespace reader  {debug::ModuleInfo debug_module{"ME.READ",    debug::LogType::Verbose};}
  namespace utility {debug::ModuleInfo debug_module{"ME.UTIL",    debug::LogType::Verbose};}
  namespace audio   {debug::ModuleInfo debug_module{"ME.AUDIO",   debug::LogType::Verbose};}
}
