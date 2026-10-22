include(GNUInstallDirs)

#install(FILES ${headers} ${PROJECT_SOURCE_DIR}/c_sources/mc_schem.hpp
#    DESTINATION ${CMAKE_INSTALL_INCLUDEDIR}
#)

get_target_property(_rust_iface mc_schem INTERFACE_LINK_LIBRARIES)   # mc_schem-shared
set_property(TARGET mc_schem PROPERTY INTERFACE_LINK_LIBRARIES
    "$<BUILD_INTERFACE:${_rust_iface}>"
    "$<INSTALL_INTERFACE:mc_schem::${_rust_iface}>")
message("_rust_iface = ${_rust_iface}")

get_target_property(mc_schem_filename ${_rust_iface} COR_FILE_NAME)
message("mc_schem_filename = ${mc_schem_filename}")

if (WIN32)
    get_target_property(mc_schem_implib_filename ${_rust_iface} COR_IMPLIB_FILE_NAME)
    set(MC_SCHEM_RUNTIME_REL ${CMAKE_INSTALL_BINDIR}/${mc_schem_filename}) #mc_schem.dll or libmc_schem.dll
    set(MC_SCHEM_LINK_REL ${CMAKE_INSTALL_LIBDIR}/${mc_schem_implib_filename}) #mc_schem.dll.lib or libmc_schem.dll.a
    install(FILES $<TARGET_FILE:${_rust_iface}>
        DESTINATION ${CMAKE_INSTALL_BINDIR}
    )
    install(FILES $<TARGET_PROPERTY:${_rust_iface},IMPORTED_IMPLIB>
        DESTINATION ${CMAKE_INSTALL_LIBDIR}
    )
else ()
    set(MC_SCHEM_RUNTIME_REL ${CMAKE_INSTALL_LIBDIR}/${mc_schem_filename}) # libmc_schem.so or libmc_schem.dylib
    set(MC_SCHEM_LINK_REL ${MC_SCHEM_RUNTIME_REL})
    install(FILES $<TARGET_FILE:${_rust_iface}>
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
    COMPATIBILITY AnyNewerVersion)

configure_package_config_file(${PROJECT_SOURCE_DIR}/cmake/mc_schem-config.cmake.in
    ${PROJECT_BINARY_DIR}/mc_schem-config.cmake
    INSTALL_DESTINATION ${CMAKE_INSTALL_LIBDIR}/cmake/mc_schem
)
install(FILES
    ${PROJECT_BINARY_DIR}/mc_schem-config-version.cmake
    ${PROJECT_BINARY_DIR}/mc_schem-config.cmake
    DESTINATION ${CMAKE_INSTALL_LIBDIR}/cmake/mc_schem
)

include(CPack)