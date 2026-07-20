/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/

#pragma once

#include "mochi/basis.hh"
#include "mochi/math/extent.hh"
#include "mochi/types.hh"



namespace mochi::rhi
{
  // Enums
  enum struct ImageTiling {
    Optimal = 0,
    Linear  = 1,
  };

  enum struct ImageType {
    e1 = 0,
    e2 = 1,
    e3 = 2
  };

  enum struct ImageUsage {
    TransferSrc                   = 0x00000001,
    TransferDst                   = 0x00000002,
    Sampled                       = 0x00000004,
    Storage                       = 0x00000008,
    ColorAttachment               = 0x00000010,
    DepthStencilAttachment        = 0x00000020,
    TransientAttachment           = 0x00000040,
    InputAttachment               = 0x00000080,
    HostTransfer                  = 0x00400000,
    VideoDecodeDst                = 0x00000400,
    VideoDecodeSrc                = 0x00000800,
    VideoDecodeDpb                = 0x00001000,
    FragmentDensityMap            = 0x00000200,
    FragmentShadingRateAttachment = 0x00000100,
    VideoEncodeDst                = 0x00002000,
    VideoEncodeSrc                = 0x00004000,
    VideoEncodeDpb                = 0x00008000,
    AttachmentFeedbackLoop        = 0x00080000,
  };
  using ImageUsageFlags = flags<ImageUsage>;



  // Interface
  struct ImageView: noncopy {
    protected:
      explicit ImageView() = default;

    public:
      virtual ~ImageView() = default;
  };

  struct ImageView2: public ImageView {
    protected:
      explicit ImageView2() = default;
  };



  struct Image: noncopy {
    protected:
      explicit Image() = default;

    public:
      virtual ~Image() = default;
  };
  
  struct Image2: Image {
    protected:
      explicit Image2() = default;
      
    public:
      virtual fun ext() const -> extent<2,u32> = 0;
      virtual fun makeView() const -> sptr<ImageView2> = 0;
  };

}

FlagEnable(mochi::rhi::ImageUsageFlags)
