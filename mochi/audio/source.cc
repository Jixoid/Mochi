/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/

#include "mochi/audio/source.hh"
#include <algorithm>

#define ef else if



namespace mochi::audio
{

  fun Source::mix_frames(float* mix_buffer, u32 frame_count, float vol_l, float vol_r) -> u32 {
    if (m_state.load() != SourceState::Playing || m_asset == nullptr) {
      return 0;
    }

    u64 cursor = m_cursor.load();
    u64 total_frames = m_asset->frame_count();
    
    if (cursor >= total_frames) {
      if (m_looping.load()) {
        cursor = 0;
      } else {
        stop();
        return 0;
      }
    }

    u32 frames_to_read = std::min<u32>(frame_count, total_frames - cursor);
    const float* src_ptr = m_asset->pcm_data.data() + (cursor * m_asset->channels);

    // Additive mixing for SIMD readiness (auto-vectorization friendly loop)
    if (m_asset->channels == 1) { // Mono to Stereo
      for (u32 i = 0; i < frames_to_read; ++i) {
        mix_buffer[i * 2 + 0] += src_ptr[i] * vol_l;
        mix_buffer[i * 2 + 1] += src_ptr[i] * vol_r;
      }
    }
    ef (m_asset->channels == 2) { // Stereo to Stereo
      for (u32 i = 0; i < frames_to_read; ++i) {
        mix_buffer[i * 2 + 0] += src_ptr[i * 2 + 0] * vol_l;
        mix_buffer[i * 2 + 1] += src_ptr[i * 2 + 1] * vol_r;
      }
    }

    m_cursor.store(cursor + frames_to_read);

    if (m_cursor.load() >= total_frames && !m_looping.load()) {
      stop();
    }

    return frames_to_read;
  }

}
