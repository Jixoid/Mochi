add_library(qaos_mochi_driver_common STATIC
  ${CMAKE_CURRENT_SOURCE_DIR}/../mochi/debug/debug.cc
)
set_target_properties(qaos_mochi_driver_common PROPERTIES POSITION_INDEPENDENT_CODE ON)
target_include_directories(qaos_mochi_driver_common PUBLIC
  "${CMAKE_CURRENT_SOURCE_DIR}/.."
  "${CMAKE_CURRENT_SOURCE_DIR}/include"
)

add_subdirectory(vulkan)
add_subdirectory(miniaudio)
