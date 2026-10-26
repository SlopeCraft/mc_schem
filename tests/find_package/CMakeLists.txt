cmake_minimum_required(VERSION 3.28)
project(mc_shem_test_find_package LANGUAGES NONE)

find_package(mc_schem CONFIG REQUIRED)

set(required_targets mc_schem::mc_schem mc_schem::mc_schem-shared mc_schem::schemtool)
foreach (target_name ${required_targets})
    if (NOT TARGET ${target_name})
        message(FATAL_ERROR "Missing target ${target_name}")
    endif ()
    get_target_property(loc ${target_name} LOCATION)
    message(STATUS "${target_name} is at ${loc}")
endforeach ()

message(STATUS "Checking variables...")
message("mc_schem_SHARED_LIB_LOCATION = ${mc_schem_SHARED_LIB_LOCATION}")
message("mc_schem_schemtool_LOCATION = ${mc_schem_schemtool_LOCATION}")