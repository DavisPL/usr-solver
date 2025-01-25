###############################################################################
# Top contributors (to current version):
#   Aina Niemetz, Mudathir Mohamed, Mathias Preiner
#
# This file is part of the cvc5 project.
#
# Copyright (c) 2009-2024 by the authors listed in the file AUTHORS
# in the top-level source directory and their institutional affiliations.
# All rights reserved.  See the file COPYING in the top-level source
# directory for licensing information.
# #############################################################################
##


####### Expanded from @PACKAGE_INIT@ by configure_package_config_file() #######
####### Any changes to this file will be overwritten by the next CMake run ####
####### The input file was cvc5Config.cmake.in                            ########

get_filename_component(PACKAGE_PREFIX_DIR "${CMAKE_CURRENT_LIST_DIR}/../../../" ABSOLUTE)

macro(set_and_check _var _file)
  set(${_var} "${_file}")
  if(NOT EXISTS "${_file}")
    message(FATAL_ERROR "File or directory ${_file} referenced by variable ${_var} does not exist !")
  endif()
endmacro()

macro(check_required_components _NAME)
  foreach(comp ${${_NAME}_FIND_COMPONENTS})
    if(NOT ${_NAME}_${comp}_FOUND)
      if(${_NAME}_FIND_REQUIRED_${comp})
        set(${_NAME}_FOUND FALSE)
      endif()
    endif()
  endforeach()
endmacro()

####################################################################################

set(CVC5_BINDINGS_JAVA ON)
set(CVC5_BINDINGS_PYTHON ON)
set(CVC5_BINDINGS_PYTHON_VERSION 3.12.4)
set(CVC5_USE_COCOA OFF)
set(CVC5_USE_CRYPTOMINISAT OFF)

if (CVC5_USE_CRYPTOMINISAT)
  find_package(Threads REQUIRED)
endif()

if(NOT TARGET cvc5::cvc5)
  include(${CMAKE_CURRENT_LIST_DIR}/cvc5Targets.cmake)
endif()

if(CVC5_BINDINGS_JAVA AND NOT TARGET cvc5::cvc5jar)
 include(${CMAKE_CURRENT_LIST_DIR}/cvc5JavaTargets.cmake)
endif()

