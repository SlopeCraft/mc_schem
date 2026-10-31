find_package(Corrosion 0.6 QUIET)

if (Corrosion_FOUND)
    return()
endif ()

include(FetchContent)

FetchContent_Declare(
    Corrosion
    GIT_REPOSITORY https://github.com/corrosion-rs/corrosion.git
    GIT_TAG 1499b14e4906a2890f5cee1547c8848db261753d #v0.6.1
    OVERRIDE_FIND_PACKAGE
    EXCLUDE_FROM_ALL
)
# Set any global configuration variables such as `Rust_TOOLCHAIN` before this line!
FetchContent_MakeAvailable(Corrosion)