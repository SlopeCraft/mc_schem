include(GNUInstallDirs)

install(FILES ${headers} ${PROJECT_SOURCE_DIR}/c_sources/mc_schem.hpp
    DESTINATION ${CMAKE_INSTALL_INCLUDEDIR}
)

if (WIN32)
    corrosion_install(TARGETS mc_schem #schemtool
        EXPORT mc_schem-targets
        RUNTIME DESTINATION ${CMAKE_INSTALL_BINDIR}
        LIBRARY DESTINATION ${CMAKE_INSTALL_BINDIR}
    )
    # Install import lib
    install(FILES $<TARGET_PROPERTY:mc_schem-shared,IMPORTED_IMPLIB>
        DESTINATION ${CMAKE_INSTALL_LIBDIR}
    )
else ()
    corrosion_install(TARGETS mc_schem #schemtool
        EXPORT mc_schem-targets
        RUNTIME DESTINATION ${CMAKE_INSTALL_BINDIR}
        LIBRARY DESTINATION ${CMAKE_INSTALL_LIBDIR}
    )
endif ()
# corrosion_install(EXPORT) just produces this file. Have to install it manually
install(FILES ${CMAKE_BINARY_DIR}/corrosion/mc_schem-targetsCorrosion.cmake
    DESTINATION ${CMAKE_INSTALL_LIBDIR}/cmake/mc_schem)

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