/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/

#include "MAdevice.hh"
#include <iostream>
#include <cstring>
#include <vector>



namespace mochi::ahi
{

  extern "C" fun MochiAHI_MakeDevice(std::string_view appName, std::array<u16, 4> appVer) -> Device* {
    return new MADevice(appName, appVer);
  }



  fun MADevice::data_callback(ma_device* pDevice, void* pOutput, const void* pInput, ma_uint32 frameCount) -> void {
    auto* device = static_cast<MADevice*>(pDevice->pUserData);
    if (device->m_user_callback) {
      device->m_user_callback(device->m_user_data, static_cast<float*>(pOutput), frameCount);
    } else {
      std::memset(pOutput, 0, frameCount * pDevice->playback.channels * sizeof(float));
    }
    (void)pInput;
  }

  MADevice::MADevice(std::string_view appName, std::array<u16, 4> appVer) {
    ma_device_config deviceConfig;

    deviceConfig = ma_device_config_init(ma_device_type_playback);
    deviceConfig.playback.format   = ma_format_f32; // Float 32-bit (Mikser için ideal)
    deviceConfig.playback.channels = 2;             // Stereo
    deviceConfig.sampleRate        = 44100;         // Standart Sample Rate
    deviceConfig.dataCallback      = data_callback;
    deviceConfig.pUserData         = this;          // Bu sınıfa erişim için

    if (ma_device_init(NULL, &deviceConfig, &m_device) != MA_SUCCESS) {
      std::cerr << "Failed to initialize Miniaudio device for " << appName << std::endl;
      m_initialized = false;
      return;
    }
    
    m_initialized = true;
  }

  MADevice::~MADevice() {
    if (m_initialized) {
      ma_device_uninit(&m_device);
    }
  }


  void MADevice::set_callback(AudioCallback callback, void* user_data) {
    m_user_callback = callback;
    m_user_data = user_data;
  }

  fun MADevice::start() -> bool {
    if (!m_initialized) return false;
    if (ma_device_start(&m_device) != MA_SUCCESS) {
      return false;
    }
    m_playing = true;
    return true;
  }

  fun MADevice::stop() -> void {
    if (!m_initialized) return;
    ma_device_stop(&m_device);
    m_playing = false;
  }

  fun MADevice::is_playing() const -> bool {
    return m_playing;
  }

  fun MADevice::decode_audio(const std::string& file_path) -> std::optional<AudioData> {
    ma_decoder decoder;
    ma_decoder_config config = ma_decoder_config_init(ma_format_f32, 2, 44100);
    
    if (ma_decoder_init_file(file_path.c_str(), &config, &decoder) != MA_SUCCESS) {
      return std::nullopt;
    }

    ma_uint64 total_frames = 0;
    ma_decoder_get_length_in_pcm_frames(&decoder, &total_frames);
    
    std::vector<float> pcm(total_frames * 2);
    ma_decoder_read_pcm_frames(&decoder, pcm.data(), total_frames, nullptr);
    ma_decoder_uninit(&decoder);

    return AudioData{44100, 2, std::move(pcm)};
  }

}
