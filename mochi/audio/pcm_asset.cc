/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/

#include "mochi/audio/pcm_asset.hh"
#include "mochi/ahi/manager/device.hh"
#include <iostream>


namespace mochi::audio
{

  PcmAsset::PcmAsset(ahi::Device* device, const std::string& file_path) {
    if (!device) return;
    
    auto decoded_opt = device->decode_audio(file_path);
    if (decoded_opt) {
      sample_rate = decoded_opt->sample_rate;
      channels = decoded_opt->channels;
      pcm_data = std::move(decoded_opt->pcm_data);
    }
    else {
      std::cerr << "Failed to load audio asset: " << file_path << "\n";
    }
  }

}
