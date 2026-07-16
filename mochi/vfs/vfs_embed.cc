/*
  This file is part of QAOS
 
  This file is licensed under the GNU General Public License version 3 (GPL3).
 
  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.
 
  Copyright (c) 2025 by Kadir Aydın.
*/


#include <filesystem>
#include <iostream>
#include <istream>
#include <string>
#include <string_view>
#include <spanstream>
#include "mochi/basis.hh"
#include "mochi/vfs/vfs.hh"

#if defined(_WIN32)
  #ifndef WIN32_LEAN_AND_MEAN
  #define WIN32_LEAN_AND_MEAN
  #endif
  #include <windows.h>
#else
  #include <dlfcn.h>
#endif



namespace vfs::__embed
{

	struct Mapped__embed: Mapped {
		public:
			Mapped__embed(void* data, u0 size) {
				m_data = data;
				m_size = size;
			}
	};


	struct Provider__embed: Provider {
		public:
			Provider__embed() {
				exists = [](Provider*, std::string_view _fpath) -> bool {
					std::string fpath(_fpath);
					for (char &C: fpath)
						if (C == '.' | C == '/' | C == '-')
							C = '_';

					void *Sym_Beg{}, *Sym_End{};

					// Platforma özgü sembol yükleme
					#if defined(_WIN32)
							HMODULE hMod = GetModuleHandle(NULL);
						Sym_Beg = (void*)GetProcAddress(hMod, ("_binary_res_"+fpath+"_start").c_str());
						Sym_End = (void*)GetProcAddress(hMod, ("_binary_res_"+fpath+"_end").c_str());
					#else
						Sym_Beg = dlsym(RTLD_DEFAULT, ("_binary_res_"+fpath+"_start").c_str());
						Sym_End = dlsym(RTLD_DEFAULT, ("_binary_res_"+fpath+"_end").c_str());
					#endif

					return (Sym_Beg && Sym_End);
				};

				open_ro = [](Provider*, std::string_view _fpath) -> sptr<std::istream> {
					std::string fpath(_fpath);
					for (char &C: fpath)
						if (C == '.' | C == '/' | C == '-')
							C = '_';

					void *Sym_Beg{}, *Sym_End{};

					// Platforma özgü sembol yükleme
					#if defined(_WIN32)
							HMODULE hMod = GetModuleHandle(NULL);
						Sym_Beg = (void*)GetProcAddress(hMod, ("_binary_res_"+fpath+"_start").c_str());
						Sym_End = (void*)GetProcAddress(hMod, ("_binary_res_"+fpath+"_end").c_str());
					#else
						Sym_Beg = dlsym(RTLD_DEFAULT, ("_binary_res_"+fpath+"_start").c_str());
						Sym_End = dlsym(RTLD_DEFAULT, ("_binary_res_"+fpath+"_end").c_str());
					#endif

					if (!Sym_Beg || !Sym_End)
						throw std::filesystem::filesystem_error("The embedded file could not be opened.", _fpath, std::make_error_code(std::errc::no_such_file_or_directory));

					auto Ret = new std::ispanstream(std::span<char>((char*)Sym_Beg, (u0)(Sym_End) - (u0)(Sym_Beg)));
					return sptr<std::istream>(Ret);
				};

				open_map = [](Provider*, std::string_view _fpath) -> sptr<Mapped> {
					std::string fpath(_fpath);
					for (char &C: fpath)
						if (C == '.' | C == '/' | C == '-')
							C = '_';

					void *Sym_Beg{}, *Sym_End{};

					// Platforma özgü sembol yükleme
					#if defined(_WIN32)
							HMODULE hMod = GetModuleHandle(NULL);
						Sym_Beg = (void*)GetProcAddress(hMod, ("_binary_res_"+fpath+"_start").c_str());
						Sym_End = (void*)GetProcAddress(hMod, ("_binary_res_"+fpath+"_end").c_str());
					#else
						Sym_Beg = dlsym(RTLD_DEFAULT, ("_binary_res_"+fpath+"_start").c_str());
						Sym_End = dlsym(RTLD_DEFAULT, ("_binary_res_"+fpath+"_end").c_str());
					#endif

					if (!Sym_Beg || !Sym_End)
						throw std::filesystem::filesystem_error("The embedded file could not be mapped.", _fpath, std::make_error_code(std::errc::no_such_file_or_directory));

					return make_sptr<Mapped__embed>(Sym_Beg, (u0)(Sym_End) - (u0)(Sym_Beg));
				};
			};
	};


	fun __attribute__((constructor(65535))) __init() { vfs::provider_reg("file", new Provider__embed()); }
	fun __attribute__((destructor(65535)))  __fini() { vfs::provider_del("file"); }

}
