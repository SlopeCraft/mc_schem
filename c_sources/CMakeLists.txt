
corrosion_import_crate(
    MANIFEST_PATH ${PROJECT_SOURCE_DIR}/Cargo.toml
    CRATE_TYPES cdylib bin
    CRATES mc_schem schemtool
    FLAGS ${MC_SCHEM_RUST_BUILD_FLAGS}
)
target_sources(mc_schem INTERFACE
    FILE_SET HEADERS
    BASE_DIRS ${PROJECT_SOURCE_DIR}/c_sources
    FILES
    ${PROJECT_SOURCE_DIR}/c_sources/mc_schem.hpp
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

if (${MSVC})
    target_compile_options(mc_schem INTERFACE $<BUILD_INTERFACE:/EHsc>)
endif ()

if (${MC_SCHEM_ENABLE_TESTS})
    add_subdirectory(unit_test)
endif ()