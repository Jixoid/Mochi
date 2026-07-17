/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#pragma once

#include "mochi/basis.hh"



namespace mochi::rhi
{
  // Enums
  enum struct SamplerAddressMode {
    Repeat            = 0,
    MirroredRepeat    = 1,
    ClampToEdge       = 2,
    ClampToBorder     = 3,
    MirrorClampToEdge = 4,
  };

  enum struct SamplerFilter {
    Nearest = 0,
    Linear  = 1,
    Cubic   = 1000015000, // requires extension
  };



  // Interface
  struct Sampler: noncopy {
    protected:
      explicit Sampler() = default;

    public:
      virtual ~Sampler() = default;
  };


  struct Sampler2: Sampler {
    protected:
      explicit Sampler2() = default;
  };

}
