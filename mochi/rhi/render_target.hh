/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/

#pragma once

#include "mochi/basis.hh"
#include "mochi/rhi/types.hh"



namespace mochi::rhi
{

  // Interface
  struct RenderTarget: noncopy {
    protected:
      explicit RenderTarget() = default;

      RenderTarget(RenderTarget&& other) noexcept = default;
      RenderTarget& operator=(RenderTarget&& other) noexcept = default;

    public:
      virtual ~RenderTarget() = default;

    public:
      virtual fun depth_format() -> Format = 0;
      virtual fun color_format() -> Format = 0;
  };

}
