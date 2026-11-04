/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/

#pragma once

#include "mochi/basis.hh"
#include "mochi/audio/types.hh"
#include "mochi/audio/source.hh"
#include <vector>
#include <mutex>



namespace mochi::audio
{

  struct Mixer: noncopy {
    public:
      Mixer() = default;

    private:
      std::vector<Source*> m_sources;
      std::mutex m_mutex;
      Listener m_listener;

    public:
      fun set_listener(const Listener& listener) -> void {
        m_listener = listener;
      }

      fun get_listener() const -> Listener {
        return m_listener;
      }

      fun add_source(Source* source) -> void;
      fun remove_source(Source* source) -> void;

      // This will be called by the AHI Device callback
      fun process(float* output_buffer, u32 frame_count) -> void;
  };

}
