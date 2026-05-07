set(TARGET_NAME ext_entt)

include(FetchContent)

FetchContent_Declare(
  entt_fetch
  GIT_REPOSITORY https://github.com/skypjack/entt.git
  GIT_TAG        v3.16.0
  GIT_SHALLOW    TRUE
  GIT_PROGRESS   TRUE
  USES_TERMINAL_DOWNLOAD TRUE
)

FetchContent_MakeAvailable(entt_fetch)

add_library(${TARGET_NAME} INTERFACE)

target_include_directories(${TARGET_NAME} INTERFACE "${entt_fetch_SOURCE_DIR}")
