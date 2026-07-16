set(TARGET_NAME test_mochi_editor)

file(GLOB_RECURSE SOURCES "src/*.cc")


# Resources
file(GLOB_RECURSE RES_FILES RELATIVE "${CMAKE_CURRENT_SOURCE_DIR}" "res/*")
set(RES_OBJECTS "")

if (NOT EMSCRIPTEN)
  foreach(RES_FILE ${RES_FILES})
    set(OBJ_FILE "${CMAKE_CURRENT_BINARY_DIR}/${RES_FILE}.o")

    string(REPLACE "/" "::" SYM_NAME "${RES_FILE}")
    string(REPLACE "." "_"  SYM_NAME "${SYM_NAME}")

    if(WIN32 OR MINGW)
      set(LD_FLAGS "-r") 
    else()
      set(LD_FLAGS "-r" "-m" "elf_x86_64")
    endif()
    
    add_custom_command(
      OUTPUT ${OBJ_FILE}
      COMMAND ${CMAKE_LINKER} ${LD_FLAGS} -b binary -o ${OBJ_FILE} ${RES_FILE}
      WORKING_DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR}
      DEPENDS ${RES_FILE}
      COMMENT "Embeding: ${RES_FILE}"
    )
    
    list(APPEND RES_OBJECTS ${OBJ_FILE})
  endforeach()
endif()



add_executable(${TARGET_NAME} ${SOURCES} ${RES_OBJECTS})

target_link_libraries(${TARGET_NAME} PRIVATE 
  qaos_qcl_shared
  qaos_qstd_shared
  qaos_qcl_platform_shared
)

target_link_options(${TARGET_NAME} PUBLIC -rdynamic)
