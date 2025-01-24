#----------------------------------------------------------------
# Generated CMake target import file for configuration "Production".
#----------------------------------------------------------------

# Commands may need to know the format version.
set(CMAKE_IMPORT_FILE_VERSION 1)

# Import target "cvc5::cvc5parser" for configuration "Production"
set_property(TARGET cvc5::cvc5parser APPEND PROPERTY IMPORTED_CONFIGURATIONS PRODUCTION)
set_target_properties(cvc5::cvc5parser PROPERTIES
  IMPORTED_LINK_DEPENDENT_LIBRARIES_PRODUCTION "cvc5::cvc5"
  IMPORTED_LOCATION_PRODUCTION "${_IMPORT_PREFIX}/lib/libcvc5parser.1.dylib"
  IMPORTED_SONAME_PRODUCTION "@rpath/libcvc5parser.1.dylib"
  )

list(APPEND _cmake_import_check_targets cvc5::cvc5parser )
list(APPEND _cmake_import_check_files_for_cvc5::cvc5parser "${_IMPORT_PREFIX}/lib/libcvc5parser.1.dylib" )

# Import target "cvc5::cvc5" for configuration "Production"
set_property(TARGET cvc5::cvc5 APPEND PROPERTY IMPORTED_CONFIGURATIONS PRODUCTION)
set_target_properties(cvc5::cvc5 PROPERTIES
  IMPORTED_LINK_DEPENDENT_LIBRARIES_PRODUCTION "Poly;Polyxx;GMP"
  IMPORTED_LOCATION_PRODUCTION "${_IMPORT_PREFIX}/lib/libcvc5.1.dylib"
  IMPORTED_SONAME_PRODUCTION "@rpath/libcvc5.1.dylib"
  )

list(APPEND _cmake_import_check_targets cvc5::cvc5 )
list(APPEND _cmake_import_check_files_for_cvc5::cvc5 "${_IMPORT_PREFIX}/lib/libcvc5.1.dylib" )

# Commands beyond this point should not need to know the version.
set(CMAKE_IMPORT_FILE_VERSION)
