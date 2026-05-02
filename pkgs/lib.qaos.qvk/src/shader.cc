/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#include "qvk/module/device.hh"
#include "qvk/core.hh"
#include "qvk/shader.hh"
#include <stdexcept>
#include <string>
#include <string_view>
#include <vulkan/vulkan_raii.hpp>
#include <sys/stat.h>
#include <sys/mman.h>
#include <fcntl.h>



namespace qvk
{

  shader::shader(core &core, std::string_view fpath, std::string_view entry)
    : vk_module(Nil)
    , m_entry(entry)
  {
    // MMap File
    int fd = open(std::string(fpath).c_str(), O_RDONLY);
    if (fd == -1)
      throw std::runtime_error("Shader dosyası açılamadı.");

    struct stat st;
    fstat(fd, &st);
    u0 size = st.st_size;

    void *data = mmap(nullptr, size, PROT_READ, MAP_PRIVATE, fd, 0);
    if (data == MAP_FAILED)
      throw std::runtime_error("Shader dosyası belleğe eşlenemdi.");
    

    // Load Shader
    vk::ShaderModuleCreateInfo info({}, size, (u32*)data);
      
    vk_module = vk::raii::ShaderModule(core.sub<device>().vdevice(), info);


    // Free File
    if (data != MAP_FAILED) munmap(data, size);
    if (fd != -1) close(fd);
  }

}
