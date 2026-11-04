/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/

#pragma once

#include "mochi/basis.hh"
#include "mochi/math/vec3.hh"



namespace mochi::audio
{

  struct Listener {
    vec3<f32> position{0.0f, 0.0f, 0.0f};
    vec3<f32> forward{0.0f, 0.0f, -1.0f};
    vec3<f32> up{0.0f, 1.0f, 0.0f};
  };

}
