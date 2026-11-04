/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#pragma once

#include "mochi/basis.hh"
#include <source_location>
#include <string_view>



namespace mochi::debug
{

  enum struct LogType: u8 { Panic = 5, Error = 4, Warning = 3, Display = 2, Log = 1, Verbose = 0 };

  struct ModuleInfo {
    public:
      ModuleInfo(const char* name, LogType loglevel)
        : m_name(name)
        , m_loglevel(loglevel)
      {}

    private:
      const char* m_name;
      LogType m_loglevel;

    public:
      fun name() const noexcept { return m_name; }
      fun loglevel() const noexcept { return m_loglevel; }
  };


  fun log(ModuleInfo &mod, LogType typ, const std::source_location location, std::string_view str) -> void;

  [[gnu::noreturn]] fun log_nr(ModuleInfo &mod, const std::source_location location, std::string_view str) -> void;
}

namespace mochi {
  namespace rhi     {extern debug::ModuleInfo debug_module;}
  namespace ahi     {extern debug::ModuleInfo debug_module;}
  namespace sys     {extern debug::ModuleInfo debug_module;}
  namespace ecs     {extern debug::ModuleInfo debug_module;}
  namespace math    {extern debug::ModuleInfo debug_module;}
  namespace asset   {extern debug::ModuleInfo debug_module;}
  namespace reader  {extern debug::ModuleInfo debug_module;}
  namespace utility {extern debug::ModuleInfo debug_module;}
  namespace audio   {extern debug::ModuleInfo debug_module;}
}


#define ME_LOG_PANIC(X, ...) mochi::debug::log_nr(debug_module, std::source_location::current(), std::format(X, ##__VA_ARGS__));

#define ME_LOG(T, X, ...) { \
  if constexpr (static_cast<u8>(T) >= _mochi_debug_level) {  \
    if (static_cast<u8>(T) >= static_cast<u8>(debug_module.loglevel())) {  \
      mochi::debug::log(debug_module, T, std::source_location::current(), std::format(X, ##__VA_ARGS__)); \
    } \
  } \
}


#define ME_LOG_ERROR(X, ...) ME_LOG(mochi::debug::LogType::Error, X, ##__VA_ARGS__)
#define ME_LOG_WARN(X, ...) ME_LOG(mochi::debug::LogType::Warning, X, ##__VA_ARGS__)
#define ME_LOG_DISP(X, ...) ME_LOG(mochi::debug::LogType::Display, X, ##__VA_ARGS__)
#define ME_LOG_LOG(X, ...) ME_LOG(mochi::debug::LogType::Log, X, ##__VA_ARGS__)
#define ME_LOG_VERB(X, ...) ME_LOG(mochi::debug::LogType::Verbose, X, ##__VA_ARGS__)

