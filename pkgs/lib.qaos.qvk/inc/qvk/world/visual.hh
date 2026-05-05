/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#pragma once

#include "Basis.hh"
#include "qvk/asset/mesh.hh"
#include "qvk/world/node.hh"
#include "qvk/rhi/pipeline.hh"
#include <vulkan/vulkan_raii.hpp>
#include <vulkan/vulkan.h>



namespace qvk
{

  /** @brief Pre-defined pipeline layout info for Physically Based Rendering (PBR). */
  extern info<pipeline> pbr_i;


  
  /** @brief Represents a renderable entity in the scene graph, associating a mesh with a pipeline. */
  struct visual: node
  {
    public:
      /**
       * @brief Construct a new visual node.
       * @param core The QVK core instance.
       * @param parent The parent node in the hierarchy.
       * @param model Initial model transform matrix.
       * @param mesh The 3D mesh to be rendered.
       * @param pipeline The graphics pipeline to use for rendering.
       */
      explicit visual(core &core, node *parent, mat4<f32> model, mesh *mesh, pipeline *pipeline);
      
    public:
      /**
       * @brief Factory method to create a new visual node.
       * @param core The QVK core instance.
       * @param parent The parent node in the hierarchy.
       * @param model Initial model transform matrix.
       * @param mesh The 3D mesh to be rendered.
       * @param pipeline The graphics pipeline to use for rendering.
       * @return Pointer to the newly created visual node.
       */
      static fun make(core &core, node *parent, mat4<f32> model, mesh *mesh, pipeline *pipeline) -> visual*;
      

    private:
      mesh *m_mesh;
      pipeline *m_pipeline;
      vk::raii::DescriptorSets m_desc_sets{nil};


    public:
      /** @brief Get the mesh associated with this visual node. */
      inline fun getMesh() { return m_mesh; }
      /**
       * @brief Set or change the mesh for this visual node.
       * @param val The new mesh.
       */
      virtual inline fun setMesh(mesh *val) -> void { m_mesh = val; }

      /** @brief Get the graphics pipeline used by this visual node. */
      inline fun getPipeline() { return m_pipeline; }

      /** @brief Access the Vulkan RAII descriptor sets allocated for this visual. */
      inline fun& getDescSets() { return m_desc_sets; }

  };

}
