# mc_schem

A rust library to generate, load, manipulate and save minecraft schematic files.

## Supported formats

|         Format          |   Extension   | Load | Save |
|:-----------------------:|:-------------:|:----:|:----:|
|       Litematica        | `.litematica` |  √   |  √   |
|    Vanilla structure    |    `.nbt`     |  √   |  √   |
| WorldEdit schem (1.13+) |   `.schem`    |  √   |  √   |
| WorldEdit schem (1.12-) | `.schematic`  |  √   |      |

## Contents

1. `mc_schem` (rlib)

   The main rust lib
2. `mc_schem` (cdylib)

   C ffi for mc_schem
3. `mc_schem` C++ wrapper

   A header-only c++ wrapper based on C ffi of mc_schem
4. `schemtool` (executable)

   An executable to do various manipulations on schematics

## Build

1. Build with cargo directly (no c/c++ files)

   ```shell
   cargo build # debug
   cargo build --release #release
   ```

2. Build with cmake (with c/c++ files)

   ```shell
   mkdir build
   cmake -S . -B build -DCMAKE_BUILD_TYPE=Release -DCMAKE_INSTALL_PREFIX=install
   cmake --build build --parallel
   cmake --install build
   ```

3. Use with cmake

    ```cmake
    find_package(mc_schem 1.3.2 CONFIG REQUIRED)
    target_link_libraries(<your target> PRIVATE mc_schem::mc_schem)
    ```
   FetchContent is also supported:
   ```cmake
   include(FetchContent)
   FetchContent_Declare(mc_schem
     GIT_REPOSITORY https://github.com/SlopeCraft/mc_schem
     GIT_TAG v1.3.2
     OVERRIDE_FIND_PACKAGE
   )
   FetchContent_MakeAvailable(mc_schem)
   ```