/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#pragma once

#include "mochi/basis.hh"
#include "mochi/types.hh"



namespace mochi::rhi
{
  // Enums
  enum struct BufferUsage: u32 {
    TransferSrc    = 0x00000001,
    TransferDst    = 0x00000002,
    UniformBuffer  = 0x00000010,
    StorageBuffer  = 0x00000020,
    IndexBuffer    = 0x00000040,
    VertexBuffer   = 0x00000080,
    IndirectBuffer = 0x00000100,
    DeviceAddress  = 0x00020000,
  };
  using BufferUsageFlags = flags<BufferUsage>;


  
  // Interface
  struct Buffer: noncopy {
    protected:
      Buffer() {};

    public:
      virtual ~Buffer() = default;
      
    public:
      virtual fun size() const -> u64 = 0;
      
      /// @brief map gpu resource to cpu
      virtual fun map() -> void = 0;
      /// @brief remove the mapping of the gpu resource to cpu
      virtual fun unmap() -> void = 0;
      /// @brief flush the cpu cache
      virtual fun flush(offs) -> void = 0;

      /// @brief return the mapped gpu resource
      virtual fun mapped() const -> void* = 0;
      /// @brief return the gpu resource pointer
      virtual fun address() const -> u64 = 0;
  };

}

FlagEnable(mochi::rhi::BufferUsage)
