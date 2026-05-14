set(TARGET_NAME test_mochi)

file(GLOB_RECURSE SOURCES "*.cc")

add_executable(${TARGET_NAME} ${SOURCES})

target_link_libraries(${TARGET_NAME} PRIVATE 
  qaos_mochi_shared
)

target_link_options(${TARGET_NAME} PUBLIC -rdynamic)
