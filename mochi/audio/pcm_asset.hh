/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/

#pragma once

#include "mochi/ahi/manager/device.hh"
#include "mochi/basis.hh"
#include <vector>



namespace mochi::audio
{

  struct Device;

  struct PcmAsset: noncopy {
    public:
      PcmAsset() = default;

      PcmAsset(u32 rate, u8 ch, std::vector<float> data)
        : sample_rate(rate), channels(ch), pcm_data(std::move(data))
      {}

      PcmAsset(ahi::Device* device, const std::string& file_path);

    public:
      u32 sample_rate{0};
      u8 channels{0};
      std::vector<float> pcm_data;

      u64 frame_count() const { 
        if (channels == 0) return 0;
        return pcm_data.size() / channels; 
      }
  };

}
