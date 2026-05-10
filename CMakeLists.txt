cmake_minimum_required(VERSION 3.28)
project(Mochi C CXX)


set(CMAKE_EXPORT_COMPILE_COMMANDS ON)

set(CMAKE_CXX_STANDARD 23)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

set(CMAKE_C_STANDARD 23)
set(CMAKE_C_STANDARD_REQUIRED ON)

add_compile_options(-gdwarf-4)

set(FETCHCONTENT_QUIET OFF)


# Debug
add_compile_definitions($<$<CONFIG:Debug>:_DBG> $<$<CONFIG:RelWithDebInfo>:_DBG>)
add_compile_definitions(_mochi_debug_ownership)


# Sanitizer
if(NOT WIN32)
  add_compile_options($<$<CONFIG:Debug>:-fsanitize=address,leak,undefined>)
  add_link_options($<$<CONFIG:Debug>:-fsanitize=address,leak,undefined>)
endif()


# LTO
include(CheckIPOSupported)
check_ipo_supported(RESULT lto_supported OUTPUT lto_error)

if(lto_supported)
  set(CMAKE_INTERPROCEDURAL_OPTIMIZATION_RELEASE ON)
  set(CMAKE_INTERPROCEDURAL_OPTIMIZATION_RELWITHDEBINFO ON)
else()
  message(WARNING "LTO is not supported: ${lto_error}")
endif()


# Thirdparty
file(GLOB childs RELATIVE "${CMAKE_CURRENT_SOURCE_DIR}/thirdparty" "${CMAKE_CURRENT_SOURCE_DIR}/thirdparty/*")
foreach(child ${childs})
  if(
    IS_DIRECTORY "${CMAKE_CURRENT_SOURCE_DIR}/thirdparty/${child}" AND
    EXISTS "${CMAKE_CURRENT_SOURCE_DIR}/thirdparty/${child}/CMakeLists.txt"
  )

    message(STATUS "Depency Project Added: ${child}")
    
    add_subdirectory("${CMAKE_CURRENT_SOURCE_DIR}/thirdparty/${child}")
      
  endif()
endforeach()


# Shader
find_program(GLSLC glslc REQUIRED)

file(GLOB_RECURSE SHADERS "mochi/shader/*.vert" "mochi/shader/*.frag")
set(SPV_FILES "")

foreach(SHADER IN LISTS SHADERS)
  get_filename_component(FILE_NAME ${SHADER} NAME)
  
  set(SPV_FILE "${CMAKE_CURRENT_SOURCE_DIR}/.qcache/${FILE_NAME}.spv")
  
  add_custom_command(
    OUTPUT ${SPV_FILE}
    COMMAND ${GLSLC} ${SHADER} -o ${SPV_FILE}
    DEPENDS ${SHADER}
    COMMENT "Vulkan Shader Compiling: ${FILE_NAME}"
  )
  
  list(APPEND SPV_FILES ${SPV_FILE})
endforeach()


# Depency
find_package(Vulkan REQUIRED)

find_package(glfw3 CONFIG QUIET)
if(glfw3_FOUND)
  set(GLFW_TARGET glfw)
  if(TARGET glfw3::glfw)
    set(GLFW_TARGET glfw3::glfw)
  endif()
else()
  find_package(GLFW REQUIRED)
  set(GLFW_TARGET glfw)
endif()

add_library(glfw_ INTERFACE)
target_link_libraries(glfw_ INTERFACE ${GLFW_TARGET})





# Engine
file(GLOB_RECURSE SOURCES "${CMAKE_CURRENT_SOURCE_DIR}/app/*.cc" "${CMAKE_CURRENT_SOURCE_DIR}/mochi/*.cc")

add_executable(qaos_mochi ${SOURCES} ${RES_OBJECTS} ${SPV_FILES})

target_link_libraries(qaos_mochi PRIVATE
  glfw_
  Vulkan::Vulkan
  ext_cgltf
  ext_stb
  ext_vma
  ext_entt
)

target_include_directories(qaos_mochi PUBLIC "${CMAKE_CURRENT_SOURCE_DIR}")

target_link_options(qaos_mochi PUBLIC -rdynamic)
