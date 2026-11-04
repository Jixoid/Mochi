/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/

#pragma once

#include "mochi/ahi/manager/device.hh"
#include "miniaudio.h"


namespace mochi::ahi
{

  struct MADevice final: Device {
    public:
      MADevice(std::string_view appName, std::array<u16, 4> appVer);
      ~MADevice() override;

    private:
      ma_device m_device;
      bool m_initialized{false};
      bool m_playing{false};

      AudioCallback m_user_callback{nullptr};
      void* m_user_data{nullptr};

      // Bu callback, donanımdan veri istendiğinde tetiklenecek.
      static fun data_callback(ma_device* pDevice, void* pOutput, const void* pInput, ma_uint32 frameCount) -> void;

    public:
      fun set_callback(AudioCallback callback, void* user_data) -> void override;

      fun start() -> bool override;
      fun stop() -> void override;
      fun is_playing() const -> bool override;

      fun decode_audio(const std::string& file_path) -> std::optional<AudioData> override;
  };

}
