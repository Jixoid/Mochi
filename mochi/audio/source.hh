/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/

#pragma once

#include "mochi/audio/pcm_asset.hh"
#include "mochi/math/vec3.hh"
#include "mochi/basis.hh"
#include <atomic>



namespace mochi::audio
{
  enum class SourceState { Stopped, Playing, Paused };

  struct Source: noncopy {
    public:
      explicit Source() = default;
      virtual ~Source() = default;

    private:
      const PcmAsset* m_asset{nullptr};
      std::atomic<u64> m_cursor{0};
      std::atomic<SourceState> m_state{SourceState::Stopped};
      std::atomic<bool> m_looping{false};

    public:
      // 3D Spatial Properties
      vec3<f32> position{0.0f, 0.0f, 0.0f};
      float volume{1.0f};
      float pitch{1.0f};
      float min_distance{1.0f};
      float max_distance{100.0f};

    public:
      fun bind_asset(const PcmAsset* asset) -> void { 
        m_asset = asset; 
        m_cursor.store(0); 
      }
      
      fun play() -> void { m_state.store(SourceState::Playing); }
      fun pause() -> void { m_state.store(SourceState::Paused); }
      fun stop() -> void { 
        m_state.store(SourceState::Stopped); 
        m_cursor.store(0); 
      }

      fun loop(bool value) -> void { m_looping.store(value); }
      fun is_looping() const -> bool { return m_looping.load(); }

      virtual fun mix_frames(float* mix_buffer, u32 frame_count, float vol_l, float vol_r) -> u32;
  };

}
