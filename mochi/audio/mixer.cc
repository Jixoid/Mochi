/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/

#include "mochi/audio/mixer.hh"
#include "mochi/math/vec3.hh"
#include <cmath>
#include <cstring>
#include <algorithm>



namespace mochi::audio
{

  fun Mixer::add_source(Source* source) -> void {
    std::lock_guard<std::mutex> lock(m_mutex);
    if (std::find(m_sources.begin(), m_sources.end(), source) == m_sources.end()) {
      m_sources.push_back(source);
    }
  }

  fun Mixer::remove_source(Source* source) -> void {
    std::lock_guard<std::mutex> lock(m_mutex);
    auto it = std::find(m_sources.begin(), m_sources.end(), source);
    if (it != m_sources.end()) {
      m_sources.erase(it);
    }
  }

  fun Mixer::process(float* output_buffer, u32 frame_count) -> void {
    std::lock_guard<std::mutex> lock(m_mutex);
    
    // Zero the master output buffer first
    std::memset(output_buffer, 0, frame_count * 2 * sizeof(float)); // Assuming stereo output (2 channels)

    vec3 listener_right = m_listener.forward.cross(m_listener.up).normalize();

    for (auto* source : m_sources) {
      // Calculate relative position
      vec3 rel_pos = source->position - m_listener.position;
      float dist = std::sqrt(rel_pos.dot(rel_pos));

      // Distance Attenuation
      float attenuation = 1.0f;
      if (dist > source->min_distance) {
        if (dist >= source->max_distance) {
          attenuation = 0.0f;
        } else {
          attenuation = 1.0f - ((dist - source->min_distance) / (source->max_distance - source->min_distance));
        }
      }

      // Panning
      float pan = 0.0f; // -1.0 (Left) to 1.0 (Right)
      if (dist > 0.0001f) {
        vec3 dir = vec3{rel_pos[0]/dist, rel_pos[1]/dist, rel_pos[2]/dist};
        pan = dir.dot(listener_right);
        pan = std::clamp(pan, -1.0f, 1.0f);
      }

      // Calculate final left/right volume multipliers
      float vol_l = source->volume * attenuation * std::clamp(1.0f - pan, 0.0f, 1.0f);
      float vol_r = source->volume * attenuation * std::clamp(1.0f + pan, 0.0f, 1.0f);

      // Tell the source to mix its data into the output buffer using the calculated volumes
      if (vol_l > 0.001f || vol_r > 0.001f) {
        source->mix_frames(output_buffer, frame_count, vol_l, vol_r);
      }
    }
  }

}
