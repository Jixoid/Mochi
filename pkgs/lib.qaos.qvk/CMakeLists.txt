set(TARGET_NAME qaos_qvk)


file(GLOB_RECURSE SOURCES "src/*.cc")
file(GLOB_RECURSE HEADERS "inc/*.hh")


find_package(Vulkan REQUIRED)


add_library(${TARGET_NAME}_shared SHARED ${SOURCES})
add_library(${TARGET_NAME}_static STATIC ${SOURCES})


target_include_directories(${TARGET_NAME}_shared PUBLIC inc)

target_link_libraries(${TARGET_NAME}_shared PUBLIC
  qaos_basis

  glfw
  Vulkan::Vulkan
)

set_target_properties(${TARGET_NAME}_shared PROPERTIES 
  OUTPUT_NAME "${TARGET_NAME}"
)



target_include_directories(${TARGET_NAME}_static PUBLIC inc)

target_link_libraries(${TARGET_NAME}_static PUBLIC
  qaos_basis

  glfw
  Vulkan::Vulkan
)

set_target_properties(${TARGET_NAME}_static PROPERTIES 
  OUTPUT_NAME "${TARGET_NAME}"
)

