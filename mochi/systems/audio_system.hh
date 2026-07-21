/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/

#pragma once

#include "mochi/basis.hh"
#include "mochi/audio/mixer.hh"
#include "mochi/ahi/manager/device.hh"
#include <thread>
#include <atomic>
#include <mutex>
#include <queue>
#include <functional>
#include <condition_variable>

namespace mochi::sys
{

  struct AudioSystem: noncopy {
    public:
      AudioSystem();
      ~AudioSystem();

    public:
      // Submit a command to be executed by the AudioWorker
      fun submit(std::function<void()> cmd) -> void;

      // Access to low level mixer (should ideally be used carefully due to threads)
      fun mixer() -> audio::Mixer& { return m_mixer; }
      fun device() -> ahi::Device* { return m_device.get(); }

    private:
      static fun master_mix_callback(void* user_data, float* output_buffer, u32 frame_count) -> void;
      fun worker_loop() -> void;

    private:
      uptr<ahi::Device> m_device;
      audio::Mixer      m_mixer;

      std::thread             m_worker;
      std::atomic<bool>       m_running{false};
      std::mutex              m_mutex;
      std::condition_variable m_cv;
      std::queue<std::function<void()>> m_queue;
  };

}
