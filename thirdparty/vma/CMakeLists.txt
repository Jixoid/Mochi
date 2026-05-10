set(TARGET_NAME ext_vma)

include(FetchContent)

FetchContent_Declare(
  vma_fetch
  GIT_REPOSITORY https://github.com/GPUOpen-LibrariesAndSDKs/VulkanMemoryAllocator.git
  GIT_TAG        v3.3.0
  GIT_SHALLOW    TRUE
  GIT_PROGRESS   TRUE
  USES_TERMINAL_DOWNLOAD TRUE
)

FetchContent_MakeAvailable(vma_fetch)

set(VMA_IMPL_SOURCE "${CMAKE_CURRENT_BINARY_DIR}/vma_impl.cc")
file(WRITE ${VMA_IMPL_SOURCE} "#define VMA_IMPLEMENTATION\n#include \"vk_mem_alloc.h\"\n")

add_library(${TARGET_NAME} STATIC ${VMA_IMPL_SOURCE})

set_target_properties(${TARGET_NAME} PROPERTIES POSITION_INDEPENDENT_CODE ON)

target_include_directories(${TARGET_NAME} PUBLIC "${vma_fetch_SOURCE_DIR}/include")

find_package(Vulkan REQUIRED)
target_link_libraries(${TARGET_NAME} PUBLIC Vulkan::Vulkan)
