set(TARGET_NAME qaos_mochi_audio_driver)

file(GLOB_RECURSE SOURCES "${CMAKE_CURRENT_SOURCE_DIR}/*.cc")

add_library(${TARGET_NAME} STATIC ${SOURCES})
set_target_properties(${TARGET_NAME} PROPERTIES POSITION_INDEPENDENT_CODE ON)
target_include_directories(${TARGET_NAME} PRIVATE "${CMAKE_CURRENT_SOURCE_DIR}/../..")
target_link_libraries(${TARGET_NAME} PRIVATE
  miniaudio
)
