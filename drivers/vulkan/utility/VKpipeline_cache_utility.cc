/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#include "drivers/vulkan/utility/VKpipeline_cache_utility.hh"
#include "mochi/debug/debug.hh"
#include "mochi/rhi/utility/pipeline_cache_utility.hh"
#include <filesystem>
#include <fstream>
#include <format>
#include <span>
#include <vector>



namespace mochi::rhi::vulkan
{

  extern "C" fun MochiRHI_MakePipelineCacheUtility(rhi::Device &device) -> rhi::PipelineCacheUtility* {
    return new VK_PipelineCacheUtility(device);
  }


  fun VK_PipelineCacheUtility::loadCache(u64 sign) -> std::vector<u8> {
    if (sign == 0)
      return {};

    auto path = std::format(".cache/mochi/vulkan/pipelines/{:x}.bin", sign);

    if (!std::filesystem::exists(path))
      return {};

    ME_LOG_VERB("pipeline cache reading")

    std::ifstream file(path, std::ios::binary | std::ios::ate);
    if (!file.is_open())
      return {};

    auto size = file.tellg();
    if (size <= 0)
      return {};

    std::vector<u8> buffer(static_cast<size_t>(size));
    file.seekg(0, std::ios::beg);
    file.read(reinterpret_cast<char*>(buffer.data()), size);
    return buffer;
  }


  fun VK_PipelineCacheUtility::saveCache(u64 sign, std::span<const u8> data) -> void {
    if (sign == 0 || data.empty())
      return;

    ME_LOG_VERB("pipeline cache writing")

    auto path = std::format(".cache/mochi/vulkan/pipelines/{:x}.bin", sign);
    auto parent = std::filesystem::path(path).parent_path();
    if (!parent.empty())
      std::filesystem::create_directories(parent);

    std::ofstream file(path, std::ios::binary);
    if (file.is_open()) {
      file.write(reinterpret_cast<const char*>(data.data()), data.size());
    }
  }

}
