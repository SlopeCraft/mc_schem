cmake_minimum_required(VERSION 3.20)
project(mc_schem VERSION 0.1.0 LANGUAGES C CXX)

option(MC_SCHEM_INSTALL "Install binaries" ON)

find_program(cargo_exe NAMES "cargo" REQUIRED)

if (CMAKE_BUILD_TYPE STREQUAL "Debug")
    set(cargo_build_mode "debug")
    set(cargo_build_flags)
else ()
    set(cargo_build_mode "release")
    set(cargo_build_flags "--release")
endif ()

set(rust_bin_dir "${CMAKE_CURRENT_SOURCE_DIR}/target/${cargo_build_mode}")

include(cmake/rust_binary_names.cmake)
rust_binary_names(shared_lib_name export_lib_name)
message(STATUS "The shared lib should be ${shared_lib_name}")
message(STATUS "The export lib should be ${export_lib_name}")

set(shared_lib_loc "${rust_bin_dir}/${shared_lib_name}")
set(export_lib_loc "${rust_bin_dir}/${export_lib_name}")

file(GLOB_RECURSE rust_sources "*.rs")
add_custom_target(mc_schem_rs
        COMMAND ${cargo_exe} build ${cargo_build_flags}
        SOURCES ${rust_sources} Cargo.toml
        COMMENT "Build all rust binaries"
        BYPRODUCTS ${shared_lib_loc} ${export_lib_loc})

file(GLOB_RECURSE headers "c_sources/*.h")
add_library(mc_schem STATIC
        c_sources/mc_schem.c
        ${headers})
add_library(mc_schem::mc_schem ALIAS mc_schem)
add_dependencies(mc_schem mc_schem_rs)
include(GenerateExportHeader)
generate_export_header(mc_schem)
target_include_directories(mc_schem INTERFACE
        $<BUILD_INTERFACE: "${CMAKE_SOURCE_DIR}/c_sources;${CMAKE_BINARY_DIR}" >
        $<INSTALL_INTERFACE:"include">)
target_link_libraries(mc_schem INTERFACE
        $<BUILD_INTERFACE:${export_lib_loc}>)

add_executable(c_test c_sources/test.c)
target_compile_features(c_test PRIVATE c_std_17)
target_link_libraries(c_test PRIVATE mc_schem)

if (${MC_SCHEM_INSTALL})
    include(CMakePackageConfigHelpers)
    write_basic_package_version_file(mc_schemConfigVersion.cmake
            VERSION ${PROJECT_VERSION}
            COMPATIBILITY AnyNewerVersion)
    install(FILES "${CMAKE_BINARY_DIR}/mc_schemConfigVersion.cmake"
            DESTINATION lib/cmake/mc_schem)

    install(FILES ${headers} "${CMAKE_BINARY_DIR}/mc_schem_export.h"
            DESTINATION include)
    install(TARGETS mc_schem
            EXPORT mc_schem-targets
            COMPONENT core
            RUNTIME DESTINATION bin
            LIBRARY DESTINATION lib)
    if (${WIN32})
        install(FILES ${shared_lib_loc}
                DESTINATION bin)
    else ()
        install(FILES ${shared_lib_loc}
                DESTINATION lib)
    endif ()
    install(FILES ${export_lib_loc}
            DESTINATION lib)

    install(EXPORT mc_schem-targets
            NAMESPACE mc_schem::
            DESTINATION lib/cmake/mc_schem)
endif ()