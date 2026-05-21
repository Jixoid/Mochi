/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#pragma once

#include "mochi/basis.hh"

#define ef else if



namespace mochi
{

  struct core;
  

  namespace module
  {
    struct bridge;
    struct display;
    struct device;
    struct renderer;
    struct memory;
  }


  namespace rhi
  {
    struct slotPush;
    struct slotDesc;
    struct slotVertex;
    
    struct listDesc;
    struct listPush;

    struct pipeline;
    struct buffer;
    struct shader;
    struct image2;

    /** @brief Information structure for a type. */
    template <typename T>
    struct info;
  }


  namespace asset
  {
    struct mesh;
    struct texture2;
    struct matrial;
  }

}
