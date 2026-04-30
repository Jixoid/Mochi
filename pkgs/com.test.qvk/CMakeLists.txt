set(TARGET_NAME com_test_qvk)



file(GLOB_RECURSE SOURCES "src/*.cc")

add_executable(${TARGET_NAME} ${SOURCES} ${RES_OBJECTS})

target_link_libraries(${TARGET_NAME} PRIVATE
  qaos_qvk_shared
)


target_link_options(${TARGET_NAME} PUBLIC -rdynamic)

set_target_properties(${TARGET_NAME} PROPERTIES 
  OUTPUT_NAME "qvk_test"
)
