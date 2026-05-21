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
  struct info<slotDesc>
  {
    public:
      info(info<buffer> ibuf, DescriptorType kind, ShaderStageFlags stage)
        : m_ibuf(ibuf)
        , m_kind(kind)
        , m_stage(stage)
      {}


    private:
      info<buffer> m_ibuf;
      DescriptorType m_kind;
      ShaderStageFlags m_stage;

    public:
      inline fun ibuf() { return &m_ibuf; }
      inline fun kind() { return m_kind; }
      inline fun stage() { return m_stage; }
  };

  

  struct slotDesc
  {
    public:
      slotDesc(info<slotDesc> info, rhi::buffer *buf)
        : m_info(info)
        , m_buf(buf)
      {
        assert(buf);
      }


    private:
      info<slotDesc> m_info;
      rhi::buffer *m_buf;
    
    public:
      inline fun info() const { return &m_info; }
      inline fun buf() const { return m_buf; }
  };

}
