/*
  This file is part of QAOS - Windows Port
*/

#include "qvk/module/device.hh"
#include "qvk/core.hh"
#include "qvk/shader.hh"
#include <stdexcept>
#include <string>
#include <string_view>
#include <vulkan/vulkan_raii.hpp>

// Windows API gereksinimleri
#ifndef WIN32_LEAN_AND_MEAN
#define WIN32_LEAN_AND_MEAN
#endif
#include <windows.h>

namespace qvk
{
    shader::shader(core &core, std::string_view fpath, std::string_view entry)
        : vk_module(Nil)
        , m_entry(entry)
    {
        // 1. Dosyayı aç (CreateFile)
        HANDLE hFile = CreateFileA(
            std::string(fpath).c_str(), 
            GENERIC_READ, 
            FILE_SHARE_READ, 
            NULL, 
            OPEN_EXISTING, 
            FILE_ATTRIBUTE_NORMAL, 
            NULL
        );

        if (hFile == INVALID_HANDLE_VALUE)
            throw std::runtime_error("Shader dosyası açılamadı (WinAPI).");

        // 2. Dosya boyutunu al
        LARGE_INTEGER fileSize;
        if (!GetFileSizeEx(hFile, &fileSize)) {
            CloseHandle(hFile);
            throw std::runtime_error("Dosya boyutu alınamadı.");
        }
        size_t size = static_cast<size_t>(fileSize.QuadPart);

        // 3. Dosya eşleme nesnesi oluştur (CreateFileMapping)
        HANDLE hMapping = CreateFileMapping(hFile, NULL, PAGE_READONLY, 0, 0, NULL);
        if (hMapping == NULL) {
            CloseHandle(hFile);
            throw std::runtime_error("Shader dosya eşleme nesnesi oluşturulamadı.");
        }

        // 4. Dosyayı belleğe eşle (MapViewOfFile -> mmap karşılığı)
        void* data = MapViewOfFile(hMapping, FILE_MAP_READ, 0, 0, 0);
        if (data == NULL) {
            CloseHandle(hMapping);
            CloseHandle(hFile);
            throw std::runtime_error("Shader dosyası belleğe eşlenemedi.");
        }

        // 5. Vulkan Shader Module yükle
        try {
            vk::ShaderModuleCreateInfo info({}, size, reinterpret_cast<uint32_t*>(data));
            vk_module = vk::raii::ShaderModule(core.sub<device>().vdevice(), info);
        } catch (...) {
            // Hata durumunda temizlik yap ve hatayı yukarı fırlat
            UnmapViewOfFile(data);
            CloseHandle(hMapping);
            CloseHandle(hFile);
            throw;
        }

        // 6. Temizlik (Memory mapping aktif kalsa bile handle'lar kapatılabilir)
        UnmapViewOfFile(data);
        CloseHandle(hMapping);
        CloseHandle(hFile);
    }
}