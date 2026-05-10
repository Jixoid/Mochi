/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#pragma once

#include "mochi/basis.hh"
#include "mochi/rhi/pipeline.hh"
#include "mochi/rhi/buffer.hh"
#include "mochi/core.hh"
#include <vulkan/vulkan_raii.hpp>



namespace mochi::asset
{

  /** @brief Represents a material that associates a pipeline with a specific descriptor set. */
  struct material
  {
    public:
      /**
       * @brief Construct a new material3d instance.
       * @param core The mochi core instance.
       * @param pip The pipeline used by this material.
       */
      explicit material(core &core, sptr<rhi::pipeline> pip);

    private:
      core &m_core;
      sptr<rhi::pipeline> m_pipeline;
      vk::raii::DescriptorSet m_desc_set{nil};

    public:
      /** @brief Access the associated pipeline. */
      inline fun& get_pipeline() { return m_pipeline; }
      
      /** @brief Access the allocated descriptor set. */
      inline fun& get_desc_set() { return m_desc_set; }

    public:
      /**
       * @brief Update the descriptor set to bind a uniform buffer.
       * @param binding The binding index defined in the shader.
       * @param buf The buffer to bind.
       */
      fun bind_uniform(u32 binding, const mochi::rhi::buffer &buf) -> void;

      /**
       * @brief Update the descriptor set to bind a texture (Combined Image Sampler).
       * @param binding The binding index defined in the shader.
       * @param image_view The image view of the texture.
       * @param sampler The sampler to use.
       */
      fun bind_texture(u32 binding, vk::ImageView image_view, vk::Sampler sampler) -> void;
  };

}
