/*
  This file is part of QAOS / Mochi

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/

#include "mochi/driver_abi.h"
#include "drivers/miniaudio/MAdevice.hh"

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

}
