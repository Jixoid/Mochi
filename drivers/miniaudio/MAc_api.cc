/*
  This file is part of QAOS / Mochi

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/

#include "mochi/driver_abi.h"
#include "drivers/miniaudio/MAdevice.hh"
#include <cstring>

using namespace mochi;

extern "C" {

MochiAudioDeviceHandle mochi_ahi_device_create(const char* app_name, uint16_t major, uint16_t minor, uint16_t patch, uint16_t build) {
  auto dev = new ahi::MADevice(app_name ? app_name : "MochiApp", {major, minor, patch, build});
  return reinterpret_cast<MochiAudioDeviceHandle>(dev);
}

void mochi_ahi_device_destroy(MochiAudioDeviceHandle device) {
  if (device) {
    delete reinterpret_cast<ahi::MADevice*>(device);
  }
}

MochiAudioDeviceHandle mochi_ahi_device_new(const char* app_name, uint16_t major, uint16_t minor, uint16_t patch, uint16_t build) {
  return mochi_ahi_device_create(app_name, major, minor, patch, build);
}

void mochi_ahi_device_dis(MochiAudioDeviceHandle device) {
  mochi_ahi_device_destroy(device);
}

void mochi_ahi_device_set_callback(MochiAudioDeviceHandle device, MochiAudioCallback callback, void* user_data) {
  if (device) {
    reinterpret_cast<ahi::MADevice*>(device)->set_callback(callback, user_data);
  }
}

bool mochi_ahi_device_start(MochiAudioDeviceHandle device) {
  if (!device) return false;
  return reinterpret_cast<ahi::MADevice*>(device)->start();
}

void mochi_ahi_device_stop(MochiAudioDeviceHandle device) {
  if (device) {
    reinterpret_cast<ahi::MADevice*>(device)->stop();
  }
}

bool mochi_ahi_device_is_playing(MochiAudioDeviceHandle device) {
  if (!device) return false;
  return reinterpret_cast<ahi::MADevice*>(device)->is_playing();
}

bool mochi_ahi_device_decode_file(MochiAudioDeviceHandle device, const char* file_path, MochiAudioData* out_data) {
  if (!device || !file_path || !out_data) return false;
  auto opt = reinterpret_cast<ahi::MADevice*>(device)->decode_audio(file_path);
  if (!opt) return false;
  out_data->sample_rate = opt->sample_rate;
  out_data->channels = opt->channels;
  out_data->frame_count = opt->pcm_data.size() / (opt->channels ? opt->channels : 1);
  out_data->pcm_data = new float[opt->pcm_data.size()];
  std::memcpy(out_data->pcm_data, opt->pcm_data.data(), opt->pcm_data.size() * sizeof(float));
  return true;
}

bool mochi_ahi_device_decode_memory(MochiAudioDeviceHandle device, const uint8_t* memory, size_t size, MochiAudioData* out_data) {
  if (!device || !memory || size == 0 || !out_data) return false;
  auto opt = reinterpret_cast<ahi::MADevice*>(device)->decode_audio_memory(memory, size);
  if (!opt) return false;
  out_data->sample_rate = opt->sample_rate;
  out_data->channels = opt->channels;
  out_data->frame_count = opt->pcm_data.size() / (opt->channels ? opt->channels : 1);
  out_data->pcm_data = new float[opt->pcm_data.size()];
  std::memcpy(out_data->pcm_data, opt->pcm_data.data(), opt->pcm_data.size() * sizeof(float));
  return true;
}

void mochi_ahi_free_audio_data(MochiAudioData* data) {
  if (data && data->pcm_data) {
    delete[] data->pcm_data;
    data->pcm_data = nullptr;
  }
}

}
