/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#pragma once

#include "Basis.hh"
#include "qvk/entity/mesh.hh"
#include "qvk/entity/node.hh"
#include "qvk/entity/pipeline.hh"
#include <vulkan/vulkan_raii.hpp>
#include <vulkan/vulkan.h>



namespace qvk
{

  extern info<pipeline> pbr_i;


  
  struct visual: node
  {
    public:
      explicit visual(core &core, node *parent, mat4<f32> model, mesh *mesh, pipeline *pipeline);
      
    public:
      static fun make(core &core, node *parent, mat4<f32> model, mesh *mesh, pipeline *pipeline) -> visual*;
      

    private:
      mesh *m_mesh;
      pipeline *m_pipeline;
      vk::raii::DescriptorSets m_desc_sets{nullptr};


    public:
      inline fun getMesh() { return m_mesh; }
      virtual inline fun setMesh(mesh *val) -> void { m_mesh = val; }

      inline fun getPipeline() { return m_pipeline; }

      inline fun& getDescSets() { return m_desc_sets; }

  };

}
