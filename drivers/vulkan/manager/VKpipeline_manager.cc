/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#include "drivers/vulkan/manager/VKpipeline_manager.hh"
#include "drivers/vulkan/VKdriver.hh"
#include "mochi/debug/debug.hh"
#include "mochi/rhi/manager/pipeline_manager.hh"
#include "mochi/vfs/vfs.hh"
#include <format>
#include <span>
#include <vector>



namespace mochi::rhi::vulkan::mng
{

  extern "C" fun MochiRHI_MakePipelineManager(rhi::mng::DeviceManager &device) -> rhi::mng::PipelineManager* {
    return new VK_PipelineManager(device);
  }


  fun VK_PipelineManager::loadCache(u64 sign) -> std::vector<u8>
  {
    if (sign == 0)
      return {};

    auto path = std::format(".cache/mochi/vulkan/pipelines/{:x}.bin", sign);

    if (!vfs::exists(path))
      return {};

    ME_LOG_VERB("pipeline cache reading")

    auto file = vfs::open_map(path);
    auto *ptr = reinterpret_cast<const u8*>(file->data());
    return std::vector<u8>(ptr, ptr + file->size());
  }


  fun VK_PipelineManager::saveCache(u64 sign, std::span<const u8> data) -> void
  {
    if (sign == 0 || data.empty())
      return;

    ME_LOG_VERB("pipeline cache writing")

    auto path = std::format(".cache/mochi/vulkan/pipelines/{:x}.bin", sign);
    vfs::open_rw(path)->write(reinterpret_cast<const char*>(data.data()), data.size()).flush();
  }

}
