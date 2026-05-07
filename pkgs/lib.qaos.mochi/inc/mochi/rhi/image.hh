/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#pragma once

#include "Basis.hh"
#include "mochi/types.hh"
#include <vulkan/vulkan_raii.hpp>
#include "vk_mem_alloc.h"



namespace mochi::rhi
{

  /** @brief Represents a Vulkan 2D image. */
  struct image2
  {
    public:
      /**
       * @brief Initialize a new image from raw pixel data.
       * @param device Logical device.
       * @param memory Memory allocator.
       * @param cmd_pool Command pool used for staging buffer copy operations.
       * @param width Width of the image.
       * @param height Height of the image.
       * @param ptr Pointer to the raw RGBA8 pixel data.
       */
      explicit image2(module::device &device, module::memory &memory, vk::raii::CommandPool &cmd_pool, u32 width, u32 height, void *ptr);
      

    private:
      VkImage m_image{VK_NULL_HANDLE};
      VmaAllocation m_allocation{nil};
      vk::raii::ImageView m_view{nil};
      vk::raii::Sampler m_sampler{nil};

      u32 m_width;
      u32 m_height;

    public:
      /** @brief Access the underlying Vulkan RAII image. */
      inline fun& image() { return m_image; }
      /** @brief Access the underlying Vulkan RAII image view. */
      inline fun& view() { return m_view; }
      /** @brief Access the underlying Vulkan RAII sampler. */
      inline fun& sampler() { return m_sampler; }
      /** @brief Get image width. */
      inline fun width() const { return m_width; }
      /** @brief Get image height. */
      inline fun height() const { return m_height; }

  };

}
