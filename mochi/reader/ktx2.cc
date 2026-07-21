/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/

#include "mochi/reader/ktx2.hh"
#include "mochi/except.hh"
// ktx.h
#include <cstring>


namespace mochi::reader
{

  template <>
  fun read<ftype::ktx>(data src, rhi::Format format) -> std::vector<u8> {
    //ktxTexture* texture;
    //KTX_error_code result;
//
    //result = ktxTexture_CreateFromMemory((ktx_uint8_t*)src.ptr(), src.size(), KTX_TEXTURE_CREATE_LOAD_IMAGE_DATA_BIT, &texture);
    //if (result != KTX_SUCCESS) {
    //  throw mochi::except("Failed to load KTX texture");
    //}
//
    //std::vector<u8> img(ktxTexture_GetDataSize(texture));
    //std::memcpy(img.data(), ktxTexture_GetData(texture), img.size());
//
    //ktxTexture_Destroy(texture);
//
    //return img;
  }

}
