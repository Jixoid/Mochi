cmake_minimum_required(VERSION 3.28)
project(QVK C CXX)


set(CMAKE_EXPORT_COMPILE_COMMANDS ON)

set(CMAKE_CXX_STANDARD 23)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

set(CMAKE_C_STANDARD 23)
set(CMAKE_C_STANDARD_REQUIRED ON)

set(CMAKE_CXX_COMPILER clang++)
set(CMAKE_C_COMPILER clang)

add_compile_options(-gdwarf-4)



# Debug
add_compile_definitions($<$<CONFIG:Debug>:_DBG> $<$<CONFIG:RelWithDebInfo>:_DBG>)
add_compile_definitions(_QVK_debug_ownership)


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


file(GLOB childs RELATIVE "${CMAKE_CURRENT_SOURCE_DIR}/pkgs" "${CMAKE_CURRENT_SOURCE_DIR}/pkgs/*")
foreach(child ${childs})
  if(
    IS_DIRECTORY "${CMAKE_CURRENT_SOURCE_DIR}/pkgs/${child}" AND
    EXISTS "${CMAKE_CURRENT_SOURCE_DIR}/pkgs/${child}/CMakeLists.txt"
  )
    
    message(STATUS "Sub Project Added: ${child}")
    add_subdirectory("${CMAKE_CURRENT_SOURCE_DIR}/pkgs/${child}")
      
  endif()
endforeach()

