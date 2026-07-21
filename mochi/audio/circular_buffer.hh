/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/

#pragma once

#include "mochi/basis.hh"
#include <atomic>
#include <vector>



namespace mochi
{

  struct CircularBuffer: noncopy {
    public:
      explicit CircularBuffer(u16 PageCount, u32 BufferSize);
      ~CircularBuffer();

    private:
      std::vector<void*> m_pages;
      u16 m_capacity{};
      alignas(64) std::atomic<u16> m_head{};
      alignas(64) std::atomic<u16> m_tail{};
    
    public:
      fun is_empty() const { return m_head.load(std::memory_order_acquire) == m_tail.load(std::memory_order_acquire); }
      fun is_full() const { return next_index(m_head.load(std::memory_order_acquire)) == m_tail.load(std::memory_order_acquire); }
      fun size() const { return (m_head.load(std::memory_order_acquire) + m_capacity - m_tail.load(std::memory_order_acquire)) % m_capacity; }
      fun capacity() const { return m_capacity -1; }

      fun write_available() const { return capacity() - size(); }
      fun read_available() const { return size(); }

    public:
      fun head() const { return m_pages[m_head.load(std::memory_order_acquire)]; }
      fun tail() const { return m_pages[m_tail.load(std::memory_order_acquire)]; }

    public:
      void advance_head() {
        u16 current = m_head.load(std::memory_order_relaxed);
        if (!is_full()) {
          m_head.store(next_index(current), std::memory_order_release);
        }
      }
      
      void advance_tail() {
        u16 current = m_tail.load(std::memory_order_relaxed);
        if (!is_empty()) {
          m_tail.store(next_index(current), std::memory_order_release);
        }
      }

      void reset() {
        m_head.store(0, std::memory_order_relaxed);
        m_tail.store(0, std::memory_order_relaxed);
      }

    private:
      fun next_index(u16 current) const noexcept -> u16 { return (current +1) % m_capacity; }
  };

}
