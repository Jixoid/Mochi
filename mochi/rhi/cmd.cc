/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#include "mochi/basis.hh"
#include "mochi/rhi/buffer.hh"
#include "mochi/rhi/convert.hh"
#include "mochi/rhi/cmd.hh"
#include "mochi/rhi/rhi.hh"
#include "mochi/rhi/pipeline.hh"
#include "mochi/rhi/listPush.hh"
#include "mochi/rhi/slotVertex.hh"
#include "vulkan/vulkan.hpp"
#include <alloca.h>
#include <vulkan/vulkan_core.h>
#include <vulkan/vulkan_raii.hpp>
#include <ranges>

#ifdef _mochi_debug_validator
  #include <iostream>
  #include <format>
#endif



namespace mochi::rhi
{

  fun cmd::setPipeline(pipeline* pipe) -> void
  {
    get().bindPipeline(VKConvert<PipelineKind>(pipe->kind()), pipe->get());
    m_cur_pipeline = pipe;
    
    m_pushConstant_index = (!pipe->info().pushConstant().empty()) ? 0 : -1;
  }




  fun cmd::writePushConstant(rhi::listPush slot) -> void
  {
    #ifdef _mochi_debug_validator
    if (m_pushConstant_index == -1)
      std::cerr << "Mochi Validation: PushConstant is undefined for this pipeline.\n";
    ef (m_pushConstant_index == -2)
      std::cerr << "Mochi Validation: PushConstant overflowed for this pipeline.\n";
    #endif 

    auto ipush = m_cur_pipeline->info().push()[m_pushConstant_index];
    
    #ifdef _mochi_debug_validator
    if (ipush.ipush().size() != slot.size())
      std::cerr << std::format("Mochi Validation: The number of data points received in PushConstant does not match the number of data points in info. [incoming: {}, info: {}].", slot.size(), ipush.ipush().size()) << '\n';
    #endif

    u32 total_size{};
    for (auto &x: (std::vector<::data>)slot) total_size += x.size();

    #ifdef _mochi_debug_validator
    if (ipush.size() != total_size)
      std::cerr << std::format("Mochi Validation: The size of the incoming data does not match the size of the info [incoming: {}, info: {}].", total_size, ipush.size()) << '\n';


    for (auto [i, x, y]: std::views::zip(std::views::iota(0), (std::vector<::data>)slot, ipush.ipush()))
      if (x.size() != y.type().size()*y.type().count())
        std::cerr << std::format("Mochi Validation: In the {} index, the size of the incoming data does not match the size of the info [incoming: {}, info: {}].", i, x.size(), y.type().size()*y.type().count()) << '\n';
    #endif


    vk::PushConstantsInfo info{
      *m_cur_pipeline->layout(),
      VKConvert<ShaderStageFlags>(ipush.stage()),
      0,
      total_size,
      slot.data().ptr()
    };

    get().pushConstants2(info);
  }


  fun cmd::bindVertexBuffers(rhi::slotVertex *vertex) -> void
  {
    #ifdef _mochi_debug_validator
    bool found{};

    for (auto &verx: m_cur_pipeline->info().vertex())
      if (verx.index() == vertex->index()) {
        found = true;
        break;
      }
    
    if (!found)
      std::cerr << "Mochi Validation: No VertexBuffer was found in this index." << std::endl;
    #endif

    
    get().bindVertexBuffers(vertex->index(), {vertex->buf()->get()}, {0});
  }

  


  fun cmd::draw(offs vertex, offs inst) -> void
  {
    get().draw(
      vertex.size(), inst.size(),
      vertex.off(),  inst.off()
    );
  }
  
  
  fun cmd::dispatch(u32 x, u32 y, u32 z) -> void
  {
    get().dispatch(x, y, z);
  }

}
