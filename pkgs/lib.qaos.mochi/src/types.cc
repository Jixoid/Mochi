/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


#include "mochi/types.hh"
#include "mochi/world/components.hh"
#include <filesystem>
#include "mochi/except.hh"
#include <string>

#if defined(__unix__) || defined(__APPLE__)
	#include <unistd.h>
  #include <fcntl.h>
  #include <sys/stat.h>
  #include <sys/mman.h>
#elif defined(_WIN32)
  #ifndef WIN32_LEAN_AND_MEAN
  #define WIN32_LEAN_AND_MEAN
  #endif
  #include <windows.h>
#endif



namespace mochi
{

  rhi::info<rhi::buffer> camera_i = rhi::info<rhi::buffer>(
    sizeof(camera_t),
    vt::make_list<
      mat4<f32>, // View
      mat4<f32>  // Projection
    >()
  );

  rhi::info<rhi::buffer> light_i = rhi::info<rhi::buffer>(
    sizeof(light_t),
    vt::make_list<
      vec4<f32>, // Position
      vec4<f32>  // Color
    >()
  );


	#if defined(__unix__) || defined(__APPLE__)

	mappedFile::mappedFile(std::string fpath) {
		
		if (!std::filesystem::is_regular_file(fpath))
			throw mochi::io_error("file not found: "+fpath+".");

		fd = open(fpath.c_str(), O_RDONLY);
		if (fd == -1)
			throw mochi::io_error("System error: " + std::to_string(errno));

		struct stat st;
		fstat(fd, &st);
		size = st.st_size;

		data = mmap(nil, size, PROT_READ, MAP_PRIVATE, fd, 0);
		if (data == MAP_FAILED)
			throw mochi::io_error("System error: " + std::to_string(errno));
	}

	mappedFile::~mappedFile() {
		if (data != MAP_FAILED) munmap(data, size);
		if (fd != -1) close(fd);
	}

	#elif defined(_WIN32)

	mappedFile::mappedFile(std::string fpath) {

		if (!std::filesystem::is_regular_file(fpath))
			throw mochi::io_error("file not found: "+fpath+".");
		
		hFile = CreateFileA(
			std::string(fpath).c_str(), 
			GENERIC_READ, 
			FILE_SHARE_READ, 
			NULL, 
			OPEN_EXISTING, 
			FILE_ATTRIBUTE_NORMAL, 
			NULL
		);

		if (hFile == INVALID_HANDLE_VALUE)
			throw mochi::io_error("Could not open shader file.");

		LARGE_INTEGER fileSize;
		if (!GetFileSizeEx(hFile, &fileSize)) {
			CloseHandle(hFile);
			throw mochi::io_error("Could not get file size.");
		}
		size = static_cast<size_t>(fileSize.QuadPart);

		hMapping = CreateFileMapping(hFile, NULL, PAGE_READONLY, 0, 0, NULL);
		if (hMapping == NULL) {
			CloseHandle(hFile);
			throw mochi::io_error("Could not create shader file mapping object.");
		}

		data = MapViewOfFile(hMapping, FILE_MAP_READ, 0, 0, 0);
		if (data == NULL) {
			CloseHandle(hMapping);
			CloseHandle(hFile);
			throw mochi::io_error("Could not map shader file to memory.");
		}
	}

	mappedFile::~mappedFile() {
		UnmapViewOfFile(data);
		CloseHandle(hMapping);
		CloseHandle(hFile);
	}

	#endif
	
}
