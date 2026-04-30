# Utterbar v0.15.6 Step 1 vendor patch.
# Replaces sherpa-onnx's `cmake/show-info.cmake` for Windows builds where
# `wmic` is no longer available (Windows 11 24H2+ removes it by default).
# This file is overlaid on top of the extracted sherpa-rs-sys/sherpa-onnx
# source by `Utterbar Windows App/scripts/build-directml-bundle.ps1`
# between cargo's first-pass extraction (which fails at the unpatched
# show-info.cmake) and the second-pass build (which uses this patched copy).
#
# The only change vs upstream:
#   * The WIN32 branch is reduced to `set(SHERPA_ONNX_OS "Windows")`.
#     The original used `wmic os get caption,version` to detect the OS
#     name + version for a cosmetic `message(STATUS ...)` line. The
#     value isn't gated on or substituted into anything else, so a
#     literal "Windows" loses no functional information.
#
# Upstream issue tracking: file PR at k2-fsa/sherpa-onnx replacing the
# wmic block with a `Get-CimInstance` PowerShell call (or a literal).
# Once accepted upstream, this patch becomes redundant and should be
# removed.

message(STATUS "CMAKE_SOURCE_DIR: ${CMAKE_SOURCE_DIR}")
message(STATUS "CMAKE_BINARY_DIR: ${CMAKE_BINARY_DIR}")
message(STATUS "PROJECT_SOURCE_DIR: ${PROJECT_SOURCE_DIR}")
message(STATUS "PROJECT_BINARY_DIR: ${PROJECT_BINARY_DIR}")
message(STATUS "CMake version: ${CMAKE_VERSION}")
message(STATUS "CMAKE_SYSTEM: ${CMAKE_SYSTEM}")
message(STATUS "CMAKE_SYSTEM_NAME: ${CMAKE_SYSTEM_NAME}")
message(STATUS "CMAKE_SYSTEM_VERSION: ${CMAKE_SYSTEM_VERSION}")
message(STATUS "CMAKE_SYSTEM_PROCESSOR: ${CMAKE_SYSTEM_PROCESSOR}")

find_package(Git QUIET)
if(Git_FOUND)
  execute_process(COMMAND
    "${GIT_EXECUTABLE}" describe --always --abbrev=40
    WORKING_DIRECTORY "${CMAKE_SOURCE_DIR}"
    OUTPUT_VARIABLE SHERPA_ONNX_GIT_SHA1
    ERROR_QUIET OUTPUT_STRIP_TRAILING_WHITESPACE
  )

  execute_process(COMMAND
    "${GIT_EXECUTABLE}" log -1 --format=%ad --date=local
    WORKING_DIRECTORY "${CMAKE_SOURCE_DIR}"
    OUTPUT_VARIABLE SHERPA_ONNX_GIT_DATE
    ERROR_QUIET OUTPUT_STRIP_TRAILING_WHITESPACE
  )
  message(STATUS "sherpa-onnx git sha1: ${SHERPA_ONNX_GIT_SHA1}")
  message(STATUS "sherpa-onnx git date: ${SHERPA_ONNX_GIT_DATE}")
else()
  message(WARNING "git is not found")
endif()

if(UNIX AND NOT APPLE)
  execute_process(COMMAND
    lsb_release -sd
    OUTPUT_VARIABLE SHERPA_ONNX_OS
    OUTPUT_STRIP_TRAILING_WHITESPACE
  )
elseif(APPLE)
  execute_process(COMMAND
    sw_vers -productName
    OUTPUT_VARIABLE _product_name
    OUTPUT_STRIP_TRAILING_WHITESPACE
  )

  execute_process(COMMAND
    sw_vers -productVersion
    OUTPUT_VARIABLE _product_version
    OUTPUT_STRIP_TRAILING_WHITESPACE
  )

  execute_process(COMMAND
    sw_vers -buildVersion
    OUTPUT_VARIABLE _build_version
    OUTPUT_STRIP_TRAILING_WHITESPACE
  )
  set(SHERPA_ONNX_OS "${_product_name} ${_product_version} ${_build_version}")
elseif(WIN32)
  # Utterbar v0.15.6 Step 1 patch: upstream uses `wmic os get caption,version`
  # which is no longer available on Windows 11 24H2+. The value is purely
  # cosmetic (only feeds the message(STATUS ...) below), so a literal
  # "Windows" is functionally equivalent.
  set(SHERPA_ONNX_OS "Windows")
else()
  set(SHERPA_ONNX_OS "Unknown")
endif()
message(STATUS "OS used to build sherpa-onnx: ${SHERPA_ONNX_OS}")

if(CMAKE_CXX_COMPILER)
  message(STATUS "C++ compiler: ${CMAKE_CXX_COMPILER}")
  if(CMAKE_CXX_COMPILER_ID)
    message(STATUS "C++ compiler ID: ${CMAKE_CXX_COMPILER_ID}")
    message(STATUS "C++ compiler version: ${CMAKE_CXX_COMPILER_VERSION}")
  endif()
endif()

if(CMAKE_C_COMPILER)
  message(STATUS "C compiler: ${CMAKE_C_COMPILER}")
  if(CMAKE_C_COMPILER_ID)
    message(STATUS "C compiler ID: ${CMAKE_C_COMPILER_ID}")
    message(STATUS "C compiler version: ${CMAKE_C_COMPILER_VERSION}")
  endif()
endif()
