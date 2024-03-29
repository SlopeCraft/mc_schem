cmake_minimum_required(VERSION 3.20)
project(mc_schem VERSION 0.1.0 LANGUAGES C CXX)

set(CMAKE_EXPORT_COMPILE_COMMANDS ON)

option(MC_SCHEM_INSTALL "Install binaries" ON)
set(MC_SCHEM_RUST_TARGET "default" CACHE STRING "Target to build rust codes. Should be consistent with C compiler. Valid values: default;guess; or any other triplet")

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
        COMMAND ${cargo_exe} build ${cargo_build_flags} --target-dir=${rust_build_dir}
        SOURCES ${rust_sources} Cargo.toml
        COMMENT "Building all rust binaries"
        BYPRODUCTS ${shared_lib_loc} ${export_lib_loc} ${schemtool_loc})

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

file(GLOB_RECURSE headers "c_sources/*.h")
add_library(mc_schem STATIC
        c_sources/mc_schem.c
        ${headers})
add_library(mc_schem::mc_schem ALIAS mc_schem)
add_dependencies(mc_schem mc_schem_rs)
include(GenerateExportHeader)
generate_export_header(mc_schem)
target_include_directories(mc_schem PUBLIC
        $<BUILD_INTERFACE: "${CMAKE_SOURCE_DIR}/c_sources;${CMAKE_BINARY_DIR}" >
        $<INSTALL_INTERFACE:"include">)
if (${link_shared_lib_directly})    # link to shared lib directly
    target_link_libraries(mc_schem PUBLIC
            $<BUILD_INTERFACE:${shared_lib_loc}>
            $<INSTALL_INTERFACE:${shared_lib_name}>)
else ()
    target_link_libraries(mc_schem PUBLIC
            $<BUILD_INTERFACE:${export_lib_loc}>
            $<INSTALL_INTERFACE:${export_lib_name}>)
endif ()

if (${CMAKE_C_COMPILER_ID} STREQUAL "MSVC")
    message(WARNING "Up til 2024/2/4, MSVC doesn't support c23, build may fail. It's suggested to use clang-cl 17+ instead")
endif ()

if (MSVC)
    target_compile_options(mc_schem PUBLIC "/Zc:__cplusplus")
    target_compile_options(mc_schem PRIVATE "/std:clatest")
    target_compile_options(mc_schem PRIVATE "/std:c17")
else ()
    target_compile_features(mc_schem PUBLIC c_std_23)
endif ()

add_library(mc_schem_hpp INTERFACE
    c_sources/mc_schem.hpp)
target_link_libraries(mc_schem_hpp INTERFACE mc_schem)
target_compile_features(mc_schem_hpp INTERFACE cxx_std_23)

add_executable(c_test c_sources/c_test.c)
target_link_libraries(c_test PRIVATE mc_schem)
if (MSVC)
    target_compile_options(c_test PRIVATE "/std:clatest")
endif ()

add_executable(cpp_test c_sources/cpp_test.cpp)
target_compile_features(cpp_test PRIVATE cxx_std_23)
target_link_libraries(cpp_test PRIVATE mc_schem_hpp)

if (${MC_SCHEM_INSTALL})
    include(CMakePackageConfigHelpers)
    write_basic_package_version_file(mc_schemConfigVersion.cmake
            VERSION ${PROJECT_VERSION}
            COMPATIBILITY AnyNewerVersion)
    install(FILES "${CMAKE_BINARY_DIR}/mc_schemConfigVersion.cmake"
            DESTINATION lib/cmake/mc_schem)

    install(FILES ${headers} "${CMAKE_BINARY_DIR}/mc_schem_export.h"
            DESTINATION include/mc_schem)
    install(TARGETS mc_schem
            EXPORT mc_schem-targets
            COMPONENT core
            RUNTIME DESTINATION bin
            LIBRARY DESTINATION lib)
    install(FILES ${schemtool_loc}
            COMPONENT schemtool
            DESTINATION bin)
    if (${WIN32})
        install(FILES ${shared_lib_loc}
                DESTINATION bin)
    else ()
        install(FILES ${shared_lib_loc}
                DESTINATION lib)
    endif ()
    if (EXISTS ${export_lib_loc})
        install(FILES ${export_lib_loc}
            DESTINATION lib)
    endif ()

    install(EXPORT mc_schem-targets
            NAMESPACE mc_schem::
            DESTINATION lib/cmake/mc_schem)

    include(CPack)
endif ()