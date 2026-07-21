/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/

#include "mochi/systems/audio_system.hh"
#include "mochi/debug/debug.hh"



namespace mochi::sys
{

  fun AudioSystem::master_mix_callback(void* user_data, float* output_buffer, u32 frame_count) -> void {
    auto* system = static_cast<AudioSystem*>(user_data);
    system->m_mixer.process(output_buffer, frame_count);
  }

  AudioSystem::AudioSystem() {
    m_device = ahi::Device::make("Mochi Audio", {1, 0, 0, 0});
    if (m_device) {
      m_device->set_callback(master_mix_callback, this);
      m_device->start();
      ME_LOG_VERB("Audio device successfully initialized and started.");
    } else {
      ME_LOG_ERROR("Failed to initialize audio device!");
    }

    m_running.store(true);
    m_worker = std::thread(&AudioSystem::worker_loop, this);
  }

  AudioSystem::~AudioSystem() {
    if (m_device) {
      m_device->stop();
    }

    m_running.store(false);
    m_cv.notify_all();

    if (m_worker.joinable()) {
      m_worker.join();
    }
    ME_LOG_VERB("AudioSystem shutdown complete.");
  }

  fun AudioSystem::submit(std::function<void()> cmd) -> void {
    {
      std::lock_guard<std::mutex> lock(m_mutex);
      m_queue.push(std::move(cmd));
    }
    ME_LOG_VERB("New audio command submitted to AudioWorker queue.");
    m_cv.notify_one();
  }

  fun AudioSystem::worker_loop() -> void {
    ME_LOG_VERB("AudioWorker thread started.");
    
    while (m_running.load()) {
      std::function<void()> cmd;
      
      {
        std::unique_lock<std::mutex> lock(m_mutex);
        m_cv.wait(lock, [this]() {
          return !m_queue.empty() || !m_running.load();
        });

        if (!m_running.load() && m_queue.empty()) {
          break;
        }

        cmd = std::move(m_queue.front());
        m_queue.pop();
      }

      if (cmd) {
        cmd();
      }
    }
    ME_LOG_VERB("AudioWorker thread stopped.");
  }

}
