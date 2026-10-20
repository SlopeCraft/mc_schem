include(GNUInstallDirs)

install(FILES ${headers} ${PROJECT_SOURCE_DIR}/c_sources/mc_schem.hpp
    DESTINATION ${CMAKE_INSTALL_INCLUDEDIR}
)
install(TARGETS mc_schem
    EXPORT mc_schem-targets
    COMPONENT core
    RUNTIME DESTINATION ${CMAKE_INSTALL_BINDIR}
    LIBRARY DESTINATION ${CMAKE_INSTALL_LIBDIR}
)
install(FILES ${schemtool_loc}
    COMPONENT schemtool
    DESTINATION ${CMAKE_INSTALL_BINDIR}
)
if (${WIN32})
    install(FILES ${shared_lib_loc}
        DESTINATION ${CMAKE_INSTALL_BINDIR}
    )
else ()
    install(FILES ${shared_lib_loc}
        DESTINATION ${CMAKE_INSTALL_LIBDIR}
    )
endif ()
if (EXISTS ${export_lib_loc})
    install(FILES ${export_lib_loc}
        DESTINATION ${CMAKE_INSTALL_LIBDIR}
    )
endif ()

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