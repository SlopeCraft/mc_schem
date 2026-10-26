include(GNUInstallDirs)

#install(FILES ${headers} ${PROJECT_SOURCE_DIR}/c_sources/mc_schem.hpp
#    DESTINATION ${CMAKE_INSTALL_INCLUDEDIR}
#)

get_target_property(rust_iface_mc_schem mc_schem INTERFACE_LINK_LIBRARIES)   # mc_schem-shared
set_property(TARGET mc_schem PROPERTY INTERFACE_LINK_LIBRARIES
    "$<BUILD_INTERFACE:${rust_iface_mc_schem}>"
    "$<INSTALL_INTERFACE:mc_schem::${rust_iface_mc_schem}>")
get_target_property(mc_schem_filename ${rust_iface_mc_schem} COR_FILE_NAME)

get_target_property(schemtool_location schemtool IMPORTED_LOCATION)
cmake_path(GET schemtool_location FILENAME schemtool_filename)

if (WIN32)
    get_target_property(mc_schem_implib_filename ${rust_iface_mc_schem} COR_IMPLIB_FILE_NAME)
    # Relative path to shared lib, dll import lib and executable in package. Required by package config file
    set(mc_schem_runtime_relative_location ${CMAKE_INSTALL_BINDIR}/${mc_schem_filename}) #mc_schem.dll or libmc_schem.dll
    set(mc_schem_import_lib_relative_location ${CMAKE_INSTALL_LIBDIR}/${mc_schem_implib_filename}) #mc_schem.dll.lib or libmc_schem.dll.a
    install(PROGRAMS $<TARGET_FILE:${rust_iface_mc_schem}> ${schemtool_location}
        DESTINATION ${CMAKE_INSTALL_BINDIR}
    )
    set(schemtool_relative_location ${CMAKE_INSTALL_BINDIR}/${schemtool_filename})
    install(FILES $<TARGET_PROPERTY:${rust_iface_mc_schem},IMPORTED_IMPLIB>
        DESTINATION ${CMAKE_INSTALL_LIBDIR}
    )
else ()
    set(mc_schem_runtime_relative_location ${CMAKE_INSTALL_LIBDIR}/${mc_schem_filename}) # libmc_schem.so or libmc_schem.dylib
    set(mc_schem_import_lib_relative_location ${mc_schem_runtime_relative_location})
    install(PROGRAMS $<TARGET_FILE:${rust_iface_mc_schem}>
        DESTINATION ${CMAKE_INSTALL_LIBDIR}
    )
    set(schemtool_relative_location ${CMAKE_INSTALL_BINDIR}/${schemtool_filename})
    install(PROGRAMS ${schemtool_location}
        DESTINATION ${CMAKE_INSTALL_BINDIR}
    )
endif ()

install(TARGETS mc_schem
    EXPORT mc_schem-targets
    FILE_SET HEADERS # This line installs mc_schem.hpp
    DESTINATION ${CMAKE_INSTALL_INCLUDEDIR}
)

install(EXPORT mc_schem-targets
    NAMESPACE mc_schem::
    DESTINATION ${CMAKE_INSTALL_LIBDIR}/cmake/mc_schem)

include(CMakePackageConfigHelpers)
write_basic_package_version_file(mc_schem-config-version.cmake
    VERSION ${PROJECT_VERSION}
    COMPATIBILITY SameMajorVersion
)

configure_package_config_file(${PROJECT_SOURCE_DIR}/cmake/mc_schem-config.cmake.in
    ${PROJECT_BINARY_DIR}/mc_schem-config.cmake
    INSTALL_DESTINATION ${CMAKE_INSTALL_LIBDIR}/cmake/mc_schem
)
install(FILES
    ${PROJECT_BINARY_DIR}/mc_schem-config-version.cmake
    ${PROJECT_BINARY_DIR}/mc_schem-config.cmake
    DESTINATION ${CMAKE_INSTALL_LIBDIR}/cmake/mc_schem
)
