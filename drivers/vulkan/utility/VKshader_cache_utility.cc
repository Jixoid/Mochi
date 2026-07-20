/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#include "drivers/vulkan/utility/VKshader_cache_utility.hh"
#include "mochi/debug/debug.hh"
#include "mochi/rhi/utility/shader_cache_utility.hh"
#include "mochi/vfs/vfs.hh"
#include <format>
#include <span>
#include <vector>



namespace mochi::rhi::vulkan
{

  extern "C" fun MochiRHI_MakeShaderCacheUtility(rhi::Device &device) -> rhi::ShaderCacheUtility* {
    return new VK_ShaderCacheUtility(device);
  }


  fun VK_ShaderCacheUtility::loadCache(u64 sign) -> std::vector<u8> {
    if (sign == 0)
      return {};

    auto path = std::format(".cache/mochi/vulkan/shaders/{:x}.bin", sign);

    if (!vfs::exists(path))
      return {};

    ME_LOG_VERB("shader cache reading")

    auto file = vfs::open_map(path);
    auto *ptr = reinterpret_cast<const u8*>(file->data());
    return std::vector<u8>(ptr, ptr + file->size());
  }


  fun VK_ShaderCacheUtility::saveCache(u64 sign, std::span<const u8> data) -> void {
    if (sign == 0 || data.empty())
      return;

    ME_LOG_VERB("shader cache writing")

    auto path = std::format(".cache/mochi/vulkan/shaders/{:x}.bin", sign);
    vfs::open_rw(path)->write(reinterpret_cast<const char*>(data.data()), data.size()).flush();
  }

}
