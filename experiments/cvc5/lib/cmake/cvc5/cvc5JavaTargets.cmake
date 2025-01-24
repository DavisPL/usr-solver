cmake_policy(PUSH)
cmake_policy(VERSION 2.8.12...3.28)

#----------------------------------------------------------------
# Generated CMake Java target import file.
#----------------------------------------------------------------

# Protect against multiple inclusion, which would fail when already imported targets are added once more.
set(_targetsDefined)
set(_targetsNotDefined)
set(_expectedTargets)
foreach(_expectedTarget cvc5jar)
  list(APPEND _expectedTargets ${_expectedTarget})
  if(TARGET ${_expectedTarget})
    list(APPEND _targetsDefined ${_expectedTarget})
  else()
    list(APPEND _targetsNotDefined ${_expectedTarget})
  endif()
endforeach()
if("%${_targetsDefined}" STREQUAL "%${_expectedTargets}")
  unset(_targetsDefined)
  unset(_targetsNotDefined)
  unset(_expectedTargets)
  cmake_policy(POP)
  return()
endif()
if(NOT "${_targetsDefined}" STREQUAL "")
  message(FATAL_ERROR
    "Some (but not all) targets in this export set were already defined.\n"
    "Targets Defined: ${_targetsDefined}\n"
    "Targets not yet defined: ${_targetsNotDefined}\n")
endif()
unset(_targetsDefined)
unset(_targetsNotDefined)
unset(_expectedTargets)

set(_prefix ${CMAKE_CURRENT_LIST_DIR}/../../../)

# Create imported target cvc5::cvc5jar
add_library(cvc5::cvc5jar IMPORTED STATIC)
set_target_properties(cvc5::cvc5jar PROPERTIES
  IMPORTED_LOCATION "${_prefix}/share/java/cvc5-1.2.0.jar"
  JAR_FILE "${_prefix}/share/java/cvc5-1.2.0.jar")


unset(_prefix)

cmake_policy(POP)
