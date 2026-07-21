/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/

#pragma once

#include "mochi/basis.hh"
#include "mochi/audio/source.hh"
#include "mochi/audio/mixer.hh"
#include <entt/entt.hpp>


namespace mochi::ecs
{

  struct AudioSource: noncopy {
    public:
      AudioSource(): source(make_uptr<audio::Source>()) {}

      AudioSource(AudioSource &&other) noexcept 
        : source(std::move(other.source)), mixer(other.mixer) 
      {
        other.mixer = nullptr;
      }

      AudioSource& operator=(AudioSource &&other) noexcept {
        if (this != &other) {
          detach_from_mixer();
          source = std::move(other.source);
          mixer = other.mixer;
          other.mixer = nullptr;
        }
        return *this;
      }

      ~AudioSource() {
        detach_from_mixer();
      }
    
    public:
      uptr<audio::Source> source;
      audio::Mixer* mixer{nullptr};


    public:
      fun attach_to_mixer(audio::Mixer &m) -> void {
        if (mixer) {
          mixer->remove_source(source.get());
        }
        mixer = &m;
        if (mixer) {
          mixer->add_source(source.get());
        }
      }

      fun detach_from_mixer() -> void {
        if (mixer && source) {
          mixer->remove_source(source.get());
          mixer = nullptr;
        }
      }
  };

}
