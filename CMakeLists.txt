cmake_minimum_required(VERSION 3.25)
project(mc_schem VERSION 1.3.0 LANGUAGES C CXX)

set(CMAKE_EXPORT_COMPILE_COMMANDS ON)

option(MC_SCHEM_INSTALL "Install binaries" ON)
option(MC_SCHEM_ENABLE_TESTS "Build the C++ wrapper unit tests" ON)
set(MC_SCHEM_RUST_TARGET "default" CACHE STRING "Target to build rust codes. Should be consistent with C compiler. Valid values: default;guess; or any other triplet")
set(MC_SCHEM_RUST_BUILD_FLAGS "" CACHE STRING "Extra flags to build rust part. Passed to cargo.")
option(MC_SCHEM_HAS_mdspan "Enable mdspan for schem. This is an option because some platforms are not very new" ON)

find_program(cargo_exe NAMES "cargo" REQUIRED)
unset(cargo_build_flags)
if (CMAKE_BUILD_TYPE STREQUAL "Debug")
    set(cargo_build_mode "debug")
    set(cargo_build_flags)
else ()
    set(cargo_build_mode "release")
    set(cargo_build_flags "--release")
endif ()

set(rust_build_dir "${CMAKE_CURRENT_BINARY_DIR}/rust-build")

include(cmake/rust_binary_names.cmake)
if (${MC_SCHEM_RUST_TARGET} STREQUAL "default")
    set(manually_assigned_target)

elseif (${MC_SCHEM_RUST_TARGET} STREQUAL "guess")
    get_suggested_cargo_target(suggested_target)
    message(STATUS "Suggested target for cargo is ${suggested_target}")
    set(cargo_build_flags ${cargo_build_flags} --target=${suggested_target})
    set(manually_assigned_target ${suggested_target})
else ()
    set(cargo_build_flags ${cargo_build_flags} --target=${MC_SCHEM_RUST_TARGET})
    set(manually_assigned_target ${MC_SCHEM_RUST_TARGET})
endif ()

if (manually_assigned_target)
    set(rust_bin_dir "${rust_build_dir}/${manually_assigned_target}/${cargo_build_mode}")
else ()
    set(rust_bin_dir "${rust_build_dir}/${cargo_build_mode}")
endif ()

message(STATUS "manually_assigned_target = ${manually_assigned_target}")

rust_binary_names("${manually_assigned_target}" shared_lib_name export_lib_name link_shared_lib_directly)
message(STATUS "The shared lib should be ${shared_lib_name}")
message(STATUS "The export lib should be ${export_lib_name}")

set(shared_lib_loc "${rust_bin_dir}/${shared_lib_name}")
if (export_lib_name)
    set(export_lib_loc "${rust_bin_dir}/${export_lib_name}")
endif ()

if (WIN32)
    set(schemtool_loc "${rust_bin_dir}/schemtool.exe")
else ()
    set(schemtool_loc "${rust_bin_dir}/schemtool")
endif ()

file(GLOB_RECURSE rust_sources "*.rs")
add_custom_target(mc_schem_rs ALL
    COMMAND ${cargo_exe} build ${cargo_build_flags} ${MC_SCHEM_RUST_BUILD_FLAGS} --target-dir=${rust_build_dir}
    SOURCES ${rust_sources} Cargo.toml
    COMMENT "Building all rust binaries"
    WORKING_DIRECTORY ${PROJECT_SOURCE_DIR}
    BYPRODUCTS ${shared_lib_loc} ${export_lib_loc} ${schemtool_loc}
)

message(STATUS "shared_lib_loc = ${shared_lib_loc}")
message(STATUS "export_lib_loc = ${export_lib_loc}")
message(STATUS "schemtool_loc = ${schemtool_loc}")
message(STATUS "link_shared_lib_directly = ${link_shared_lib_directly}")

if (${WIN32})
    cmake_path(RELATIVE_PATH rust_bin_dir BASE_DIRECTORY ${CMAKE_CURRENT_BINARY_DIR} OUTPUT_VARIABLE relative_path_to_rs_build_dir)
    string(REPLACE "/" "\\" relative_path_to_rs_build_dir ${relative_path_to_rs_build_dir})
    add_custom_target(make_dll_symlink
            COMMAND mklink mc_schem.dll ".\\${relative_path_to_rs_build_dir}\\mc_schem.dll"
            WORKING_DIRECTORY ${CMAKE_CURRENT_BINARY_DIR})
endif ()

add_library(mc_schem INTERFACE)
target_sources(mc_schem INTERFACE
    $<BUILD_INTERFACE:${PROJECT_SOURCE_DIR}/c_sources/mc_schem.hpp>
)
add_dependencies(mc_schem mc_schem_rs)

include(GNUInstallDirs)
target_include_directories(mc_schem INTERFACE
    $<BUILD_INTERFACE:${PROJECT_SOURCE_DIR}/c_sources>
    $<INSTALL_INTERFACE:${CMAKE_INSTALL_INCLUDEDIR}>
)
if (${link_shared_lib_directly})    # link to shared lib directly
    target_link_libraries(mc_schem INTERFACE
            $<BUILD_INTERFACE:${shared_lib_loc}>
            $<INSTALL_INTERFACE:${shared_lib_name}>)
    target_link_directories(mc_schem INTERFACE
        $<INSTALL_INTERFACE:${CMAKE_INSTALL_LIBDIR}>
        $<INSTALL_INTERFACE:${CMAKE_INSTALL_BINDIR}>
    )
else ()
    target_link_libraries(mc_schem INTERFACE
            $<BUILD_INTERFACE:${export_lib_loc}>
            $<INSTALL_INTERFACE:${export_lib_name}>)
    target_link_directories(mc_schem INTERFACE
        $<INSTALL_INTERFACE:${CMAKE_INSTALL_LIBDIR}>
    )
endif ()

target_compile_features(mc_schem INTERFACE cxx_std_23)
if (${MC_SCHEM_HAS_mdspan})
    target_compile_definitions(mc_schem INTERFACE MC_SCHEM_HAS_mdspan=1)
endif ()

if (NOT ${PROJECT_SOURCE_DIR} STREQUAL ${CMAKE_SOURCE_DIR})
    # Support fetched mode
    add_library(mc_schem::mc_schem ALIAS mc_schem)
    set(mc_schem_SHARED_LIB_FILENAME ${shared_lib_name} PARENT_SCOPE)
    set(mc_schem_SHARED_LIB_LOCATION ${shared_lib_loc} PARENT_SCOPE)
endif ()

if (${MC_SCHEM_ENABLE_TESTS})
    enable_testing()
    add_subdirectory(c_sources/unit_test)
endif ()

if (${MC_SCHEM_INSTALL})
    include(cmake/install.cmake)
endif ()