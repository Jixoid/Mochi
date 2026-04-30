/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#pragma once

#include "Basis.hh"
#include "qvk/meta.hh"
#include "qvk/types.hh"
#include <vulkan/vulkan_raii.hpp>



namespace qvk
{

  struct pipeline
  {
    public:
      explicit pipeline(qvk::engine &engine, qvk::shader vert, qvk::shader frag);

    public:
      static fun make(qvk::engine &engine, qvk::shader vert, qvk::shader frag, metaobj<pipeline> *meta = Nil) -> pipeline*;


    private:
      vk::raii::PipelineLayout vk_layout;
      vk::raii::Pipeline       vk_pipeline;

    public:
      inline fun& get() { return vk_pipeline; }
      inline fun& layout() { return vk_layout; }
  };
  
}
