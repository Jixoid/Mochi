/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/

#include "mochi/audio/circular_buffer.hh"
#include "mochi/basis.hh"
#include <cstdlib>
#include <new>



namespace mochi
{

  CircularBuffer::CircularBuffer(u16 PageCount, u32 BufferSize) {
    u16 actual_pages = PageCount+1;
    m_capacity = actual_pages;
    
    std::vector<void*> pages;
    pages.reserve(actual_pages);

    for (i32 i = 0; i < actual_pages; i++) {
      void* blok = aligned_alloc(32, BufferSize);

      if (blok == nullptr) [[unlikely]]
        throw std::bad_alloc();

      pages.push_back(blok);
    }

    m_pages = std::move(pages);

    m_head = m_tail = 0;
  }

  CircularBuffer::~CircularBuffer() {
    for (auto page: m_pages) {
      free(page);
    }
  }

}
