/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#pragma once

#include "basis.hh"
#include "mochi/rhi/image.hh"
#include <vulkan/vulkan_raii.hpp>
#include <vulkan/vulkan.h>



namespace mochi::asset
{

  /** @brief Represents a 2D texture asset loaded into a Vulkan image. */
  struct texture2
  {
    public:
      /**
       * @brief Construct a 2D texture directly from an existing Vulkan image.
       * @param core The mochi core instance.
       * @param data The Vulkan image containing the texture data.
       */
      explicit texture2(core &core, sptr<rhi::image2> data);

      /**
       * @brief Construct a 2D texture by loading an image file from disk.
       * @param core The mochi core instance.
       * @param fpath The file path to the image.
       */
      explicit texture2(core &core, const std::string &fpath);

    public:
      /**
       * @brief Factory method to create a 2D texture from an existing Vulkan image.
       * @param core The mochi core instance.
       * @param data The Vulkan image containing the texture data.
       * @return Pointer to the newly created texture.
       */
      static fun make(core &core, sptr<rhi::image2> data) -> sptr<texture2>;

      /**
       * @brief Factory method to load and create a 2D texture from a file.
       * @param core The mochi core instance.
       * @param fpath The file path to the image.
       * @return Pointer to the newly created texture.
       */
      static fun make(core &core, const std::string &fpath) -> sptr<texture2>;


    private:
      sptr<rhi::image2> m_data{nil};

    public:
      /** @brief Access the underlying Vulkan image holding the texture data. */
      inline fun data() { return m_data; }
  };

}
