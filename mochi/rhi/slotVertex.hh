/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#pragma once

#include "mochi/basis.hh"
#include "mochi/rhi/rhi.hh"
#include "mochi/rhi/buffer.hh"
#include "mochi/types.hh"
#include <cassert>



namespace mochi::rhi
{

  template<>
  struct info<slotVertex>
  {
    public:
      info(info<rhi::buffer> ibuf, VertexInputRate inputRate, i16 index = -1)
        : m_ibuf(ibuf)
        , m_inputRate(inputRate)
        , m_index(index)
      {}


    private:
      info<rhi::buffer> m_ibuf;
      i16 m_index;
      VertexInputRate m_inputRate;

    public:
      inline fun  ibuf() const { return &m_ibuf; }
      inline fun  inputRate() const { return m_inputRate; }
      inline fun& index() { return m_index; }
  };



  struct slotVertex
  {
    public:
      slotVertex(info<slotVertex> info, rhi::buffer *buf, i16 index = -1)
        : m_info(info)
        , m_buf(buf)
        , m_index(index)
      {
        assert(buf);
      }


    private:
      info<slotVertex> m_info;
      i16 m_index;
      rhi::buffer *m_buf;
    
    public:
      inline fun index() const { return m_index; }
      inline fun info() const { return &m_info; }
      inline fun buf() const { return m_buf; }
  };

}
