set(TARGET_NAME qaos_mochi_graphic_driver)

file(GLOB_RECURSE SOURCES "${CMAKE_CURRENT_SOURCE_DIR}/*.cc")


find_package(Vulkan REQUIRED)
pkg_check_modules(SHADERC REQUIRED IMPORTED_TARGET shaderc)

add_library(${TARGET_NAME} STATIC ${SOURCES})
set_target_properties(${TARGET_NAME} PROPERTIES POSITION_INDEPENDENT_CODE ON)
target_include_directories(${TARGET_NAME} PUBLIC "${CMAKE_CURRENT_SOURCE_DIR}/../..")
target_link_libraries(${TARGET_NAME} PUBLIC
  Vulkan::Vulkan
  PkgConfig::SHADERC
  VulkanMemoryAllocator
)
