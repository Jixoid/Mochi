/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/

#pragma once

#include "mochi/basis.hh"
#include <array>
#include <string_view>
#include <string>
#include <vector>
#include <optional>


namespace mochi::ahi
{

  struct AudioData {
    u32 sample_rate;
    u8 channels;
    std::vector<float> pcm_data;
  };

  struct Device;


  // External
  extern "C" fun MochiAHI_MakeDevice(std::string_view appName, std::array<u16, 4> appVer) -> Device*;


  // Interface
  struct Device: noncopy {
    protected:
      Device() = default;

    public:
      virtual ~Device() = default;

      static fun make(std::string_view appName, std::array<u16, 4> appVer) {
        return make_uptr(MochiAHI_MakeDevice(appName, appVer));
      }

    public:
      using AudioCallback = void(*)(void* user_data, float* output_buffer, u32 frame_count);
      virtual fun set_callback(AudioCallback callback, void* user_data) -> void = 0;

      // Audio Control
      virtual fun start() -> bool = 0;
      virtual fun stop() -> void = 0;

      // Status
      virtual fun is_playing() const -> bool = 0;

      // Asset Loading via Decoder
      virtual fun decode_audio(const std::string& file_path) -> std::optional<AudioData> = 0;
  };

}
