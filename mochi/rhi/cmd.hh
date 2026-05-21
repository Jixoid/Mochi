/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#pragma once

#include "mochi/basis.hh"
#include "mochi/rhi/listPush.hh"
#include "mochi/rhi/pipeline.hh"
#include "mochi/rhi/slotVertex.hh"
#include <vulkan/vulkan_core.h>
#include <vulkan/vulkan_raii.hpp>



namespace mochi::rhi
{

  struct cmd
  {
    public:
      inline cmd(vk::raii::CommandBuffer *cmd)
        : m_cmd(cmd)
      {}
      

    private:
      vk::raii::CommandBuffer *m_cmd{};

    public:
      inline fun& get() { return *m_cmd; }


    private:
      pipeline *m_cur_pipeline{};
      i16 m_pushConstant_index{-1};
      i16 m_vertexBuffer_index{-1};


    public:
      fun setPipeline(pipeline* pipe) -> void;
      
      fun writePushConstant(rhi::listPush slot) -> void;
      fun bindVertexBuffers(rhi::slotVertex *vextex) -> void;
      
      fun draw(offs vertex, offs inst) -> void;
      fun dispatch(u32 x, u32 y, u32 z) -> void;
  };

}
