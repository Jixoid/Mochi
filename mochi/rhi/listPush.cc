/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#include "mochi/basis.hh"
#include "mochi/types.hh"
#include "mochi/rhi/rhi.hh"
#include "mochi/rhi/slotPush.hh"
#include "mochi/rhi/listPush.hh"
#include <cassert>



namespace mochi::rhi
{

  constexpr inline fun align_size(u64 size, u64 alignment) { if (alignment == 0) return size; else return ((size + alignment - 1) / alignment) * alignment; }

  info<listPush>::info(ShaderStageFlags stage, std::vector<info<slotPush>> ipush)
    : m_stage(stage)
    , m_ipush(ipush)
  {
    u32 off{};

    for (auto &isl: ipush) {
      off = align_size(off, isl.type().align());

      isl.offset() = off;

      off += isl.type().size() * isl.type().count();
    }

    m_size = off;
  }

}
