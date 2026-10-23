cmake_minimum_required(VERSION 3.25)
project(mc_schem VERSION 1.3.1 LANGUAGES C CXX)

set(CMAKE_EXPORT_COMPILE_COMMANDS ON)

option(MC_SCHEM_INSTALL "Install binaries" ON)
option(MC_SCHEM_ENABLE_TESTS "Build the C++ wrapper unit tests" ON)
#set(MC_SCHEM_RUST_TARGET "default" CACHE STRING "Target to build rust codes. Should be consistent with C compiler. Valid values: default;guess; or any other triplet")
set(MC_SCHEM_RUST_BUILD_FLAGS "" CACHE STRING "Extra flags to build rust part. Passed to cargo.")
option(MC_SCHEM_HAS_mdspan "Enable mdspan for schem. This is an option because some platforms are not very new" ON)

include(GNUInstallDirs)

include(cmake/corrosion.cmake)
find_package(Corrosion REQUIRED)

corrosion_import_crate(
    MANIFEST_PATH ${PROJECT_SOURCE_DIR}/Cargo.toml
    CRATE_TYPES cdylib bin
    CRATES mc_schem schemtool
    FLAGS ${MC_SCHEM_RUST_BUILD_FLAGS}
)
target_sources(mc_schem INTERFACE
    FILE_SET HEADERS
    BASE_DIRS c_sources
    FILES
    c_sources/mc_schem.hpp
)
target_include_directories(mc_schem INTERFACE
    $<BUILD_INTERFACE:${PROJECT_SOURCE_DIR}/c_sources>
    $<INSTALL_INTERFACE:${CMAKE_INSTALL_INCLUDEDIR}>
)
target_link_directories(mc_schem INTERFACE
    $<INSTALL_INTERFACE:${CMAKE_INSTALL_LIBDIR}>
)
target_compile_features(mc_schem INTERFACE cxx_std_23)
if (${MC_SCHEM_HAS_mdspan})
    target_compile_definitions(mc_schem INTERFACE MC_SCHEM_HAS_mdspan=1)
endif ()


if (${MC_SCHEM_ENABLE_TESTS})
    enable_testing()
    add_subdirectory(c_sources/unit_test)
endif ()

if (NOT ${PROJECT_SOURCE_DIR} STREQUAL ${CMAKE_SOURCE_DIR})
    add_library(mc_schem::mc_schem ALIAS mc_schem)
    # Corrosion setup target properties with DEFER(something like run at exiting scope). So we also have to use DEFER,
    # this will be executed after Corrosion's defer, so can get properties
    cmake_language(DEFER CALL mc_schem_export_shared_lib_location)
    macro(mc_schem_export_shared_lib_location)
        get_target_property(loc mc_schem-shared IMPORTED_LOCATION)
        message(STATUS "mc_schem_SHARED_LIB_LOCATION = ${loc}")
        # This variable is useful for deploying
        set(mc_schem_SHARED_LIB_LOCATION ${loc} PARENT_SCOPE)
    endmacro()
endif ()

if (${MC_SCHEM_INSTALL})
    include(cmake/install.cmake)
endif ()

