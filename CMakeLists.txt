cmake_minimum_required(VERSION 3.25)
project(mc_schem VERSION 1.3.2 LANGUAGES C CXX)

set(CMAKE_EXPORT_COMPILE_COMMANDS ON)

option(MC_SCHEM_INSTALL "Install binaries" ON)
option(MC_SCHEM_ENABLE_TESTS "Build the C++ wrapper unit tests" ON)
option(MC_SCHEM_ENABLE_EXAMPLES "Build C++ examples" ON)
#set(MC_SCHEM_RUST_TARGET "default" CACHE STRING "Target to build rust codes. Should be consistent with C compiler. Valid values: default;guess; or any other triplet")
set(MC_SCHEM_RUST_BUILD_FLAGS "" CACHE STRING "Extra flags to build rust part. Passed to cargo.")
option(MC_SCHEM_HAS_mdspan "Enable mdspan for schem. This is an option because some platforms are not very new" ON)
option(MC_SCHEM_HAS_boost "Enable boost for schem. Mostly for multiarray" ON)

if (${MC_SCHEM_ENABLE_TESTS})
    enable_testing()
endif ()

include(GNUInstallDirs)

include(cmake/3rdParty/corrosion.cmake)
find_package(Corrosion REQUIRED)

if (${MC_SCHEM_HAS_boost})
    include(cmake/3rdParty/boost.cmake)
endif ()

add_subdirectory(c_sources)

get_target_property(loc mc_schem-shared IMPORTED_LOCATION)
get_target_property(loc_schemtool schemtool IMPORTED_LOCATION)
if (NOT ${PROJECT_SOURCE_DIR} STREQUAL ${CMAKE_SOURCE_DIR})
    add_library(mc_schem::mc_schem ALIAS mc_schem)
    add_library(mc_schem::mc_schem-shared ALIAS mc_schem)
    add_executable(mc_schem::schemtool ALIAS schemtool)

    message(STATUS "mc_schem_SHARED_LIB_LOCATION = ${loc}")
    message(STATUS "mc_schem_schemtool_LOCATION = ${loc_schemtool}")
    # This variable is useful for deploying
    set(mc_schem_SHARED_LIB_LOCATION ${loc} PARENT_SCOPE)
    set(mc_schem_schemtool_LOCATION ${loc_schemtool} PARENT_SCOPE)
endif ()

if (${MC_SCHEM_ENABLE_EXAMPLES})
    add_subdirectory(examples)
endif ()

if (${MC_SCHEM_INSTALL})
    include(cmake/install.cmake)
endif ()

if (${PROJECT_SOURCE_DIR} STREQUAL ${CMAKE_SOURCE_DIR})
    include(CPack)
endif ()