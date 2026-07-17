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
add_compile_options(-Wno-gnu-alignof-expression)
add_compile_definitions($<$<CONFIG:Debug>:_DBG> $<$<CONFIG:RelWithDebInfo>:_DBG>)
add_compile_definitions(_mochi_debug_khronos_validator)
add_compile_definitions(_mochi_debug_validator)
add_compile_definitions(_mochi_debug_level=0)
add_compile_definitions(_mochi_driver_debug_statistics)


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


# Resources
file(GLOB_RECURSE ABS_RES_FILES "${CMAKE_CURRENT_SOURCE_DIR}/mochi/res/*")
set(RES_OBJECTS "")

if (NOT EMSCRIPTEN)
  foreach(ABS_RES_FILE ${ABS_RES_FILES})
    
    file(RELATIVE_PATH RES_FILE "${CMAKE_CURRENT_SOURCE_DIR}/mochi" "${ABS_RES_FILE}")
    
    set(OBJ_FILE "${CMAKE_CURRENT_BINARY_DIR}/${RES_FILE}.o")

    if(WIN32 OR MINGW)
      set(LD_FLAGS "-r") 
    else()
      set(LD_FLAGS "-r" "-m" "elf_x86_64")
    endif()
    
    add_custom_command(
      OUTPUT ${OBJ_FILE}
      COMMAND ${CMAKE_LINKER} ${LD_FLAGS} -b binary -o ${OBJ_FILE} ${RES_FILE}
      WORKING_DIRECTORY "${CMAKE_CURRENT_SOURCE_DIR}/mochi"
      DEPENDS "${ABS_RES_FILE}"
      COMMENT "Embedding: ${RES_FILE}"
    )
    
    list(APPEND RES_OBJECTS ${OBJ_FILE})
  endforeach()
endif()


# Depency
find_package(PkgConfig REQUIRED)

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

pkg_check_modules(SHADERC REQUIRED IMPORTED_TARGET shaderc)



# Engine
file(GLOB_RECURSE SOURCES "${CMAKE_CURRENT_SOURCE_DIR}/mochi/*.cc")
file(GLOB_RECURSE DRIVER_SOURCES "${CMAKE_CURRENT_SOURCE_DIR}/drivers/vulkan/*.cc")


add_library(qaos_mochi_shared SHARED ${SOURCES} ${RES_OBJECTS} ${DRIVER_SOURCES})
target_include_directories(qaos_mochi_shared PUBLIC "${CMAKE_CURRENT_SOURCE_DIR}")
target_link_libraries(qaos_mochi_shared PUBLIC
  glfw_
  Vulkan::Vulkan
  PkgConfig::SHADERC
  ext_cgltf
  ext_stb
  VulkanMemoryAllocator
  EnTT
)


add_library(qaos_mochi_static STATIC ${SOURCES} ${RES_OBJECTS} ${DRIVER_SOURCES})
target_include_directories(qaos_mochi_static PUBLIC "${CMAKE_CURRENT_SOURCE_DIR}")
target_link_libraries(qaos_mochi_static PUBLIC
  glfw_
  Vulkan::Vulkan
  PkgConfig::SHADERC
  ext_cgltf
  ext_stb
  VulkanMemoryAllocator
  EnTT
)



# Test App
add_subdirectory(app)

# Editor
add_subdirectory(editor)

