/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#include "mochi/basis.hh"
#include "drivers/vulkan/VKconvert.hh"
#include "drivers/vulkan/VKcommand.hh"
#include "drivers/vulkan/VKpipeline.hh"
#include "mochi/rhi/pipeline.hh"



namespace mochi::rhi::vulkan
{

  fun VK_Command::bindPipeline(Pipeline *pipe) -> void {
    get().bindPipeline(
      VKConvert<PipelineKind>(pipe->kind()),
      static_cast<VK_Pipeline*>(pipe)->get()
    );

    m_pipe = static_cast<VK_Pipeline*>(pipe);
  };


  fun VK_Command::pushConstant(ShaderStageFlags stage, u32 off, data dat) -> void {
    vk::PushConstantsInfo info{
      m_pipe->layout(),
      VKConvert<ShaderStageFlags>(stage),
      off,
      (u32)dat.size(),
      dat.ptr()
    };

    get().pushConstants2(info);
  }

  
  fun VK_Command::draw(offs vertex, offs inst) -> void {
    get().draw(
      vertex.size(), inst.size(),
      vertex.off(),  inst.off()
    );
  }
  
  
  fun VK_Command::dispatch(extent<3,u32> e) -> void {
    get().dispatch(e.x(), e.y(), e.z());
  }


  fun VK_Command::setViewport(u32 first, std::vector<Viewport> viewports) -> void {
    std::vector<vk::Viewport> vps;
    vps.reserve(viewports.size());
    
    for (auto vp: viewports)
      vps.push_back(vk::Viewport(
        vp.min()[0], vp.min()[1],
        vp.max()[0]-vp.min()[0], vp.max()[1]-vp.min()[1],
        vp.min()[2], vp.max()[2]
      ));

    get().setViewport(first, vps);
  }


  fun VK_Command::setScissor(u32 first, std::vector<Rect2D> scissors) -> void {
    std::vector<vk::Rect2D> scs;
    scs.reserve(scissors.size());
    
    for (auto sc: scissors)
      scs.push_back(vk::Rect2D(
        {sc.offset()[0], sc.offset()[1]},
        {sc.extent()[0], sc.extent()[1]}
      ));

    get().setScissor(first, scs);
  }

}
