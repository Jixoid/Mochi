set(TARGET_NAME ext_stb)

include(FetchContent)

FetchContent_Declare(
  stb_fetch
  GIT_REPOSITORY https://github.com/nothings/stb.git
  GIT_TAG        master
  GIT_SHALLOW    TRUE
  GIT_PROGRESS   TRUE
  USES_TERMINAL_DOWNLOAD TRUE
)

FetchContent_MakeAvailable(stb_fetch)

set(STB_IMPL_SOURCE "${CMAKE_CURRENT_BINARY_DIR}/stb_image.c")
file(WRITE ${STB_IMPL_SOURCE} "#define STB_IMAGE_IMPLEMENTATION\n#include \"stb_image.h\"\n")


add_library(${TARGET_NAME} STATIC ${STB_IMPL_SOURCE})

set_target_properties(${TARGET_NAME} PROPERTIES POSITION_INDEPENDENT_CODE ON)

target_include_directories(${TARGET_NAME} PUBLIC "${stb_fetch_SOURCE_DIR}")
