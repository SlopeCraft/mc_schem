function(rust_binary_names out_var_shared_lib out_var_export_lib out_var_link_shared_lib)
    unset(${out_var_shared_lib} PARENT_SCOPE)
    unset(${out_var_export_lib} PARENT_SCOPE)
    unset(${out_var_link_shared_lib} PARENT_SCOPE)

    if (WIN32)
        #if (MSVC)
        set(${out_var_shared_lib} "mc_schem.dll" PARENT_SCOPE)
        set(${out_var_export_lib} "mc_schem.dll.lib" PARENT_SCOPE)
        set(${out_var_link_shared_lib} OFF PARENT_SCOPE)
        return()
        #endif ()


        return()
    endif ()


    if (LINUX)
        set(${out_var_shared_lib} "libmc_schem.so" PARENT_SCOPE)
        set(${out_var_link_shared_lib} ON PARENT_SCOPE)
        return()
    endif ()


    if (DARWIN)
        set(${out_var_shared_lib} "libmc_schem.dylib" PARENT_SCOPE)
        set(${out_var_link_shared_lib} ON PARENT_SCOPE)
        return()
    endif ()

    message(FATAL_ERROR "Can not guess the names of generated binaries.")
endfunction(rust_binary_names)


function(get_clang_target)
    include(CMakeParseArguments)
    cmake_parse_arguments(GCT "" "OUT_VAR_TARGET;OUT_VAR_THREAD_MODEL;CLANG_COMPILER" "" ${ARGN})

    set(clang_exe ${CMAKE_C_COMPILER})
    if (GCT_CLANG_COMPILER)
        set(clang_exe ${GCT_CLANG_COMPILER})
    endif ()

    execute_process(COMMAND ${clang_exe} "--version"
            OUTPUT_VARIABLE out_string
            COMMAND_ERROR_IS_FATAL ANY)

    string(REPLACE "\n" ";" out_string ${out_string})
    message("out_string = ${out_string}")

    if (GCT_OUT_VAR_TARGET)
        unset(${GCT_OUT_VAR_TARGET} PARENT_SCOPE)
        unset(_target)
        foreach (str ${out_string})
            if (${str} MATCHES "Target: ")
                string(SUBSTRING ${str} 8 -1 _target)
                break()
            endif ()
        endforeach ()
        if (NOT _target)
            message(FATAL_ERROR "Failed to parse target of C compiler from output \"${out_string}\"")
        endif ()
        set(${GCT_OUT_VAR_TARGET} ${_target} PARENT_SCOPE)
    endif ()

    if (GCT_OUT_VAR_THREAD_MODEL)
        unset(${GCT_OUT_VAR_THREAD_MODEL} PARENT_SCOPE)
        unset(_thread_model)
        foreach (str ${out_string})
            if (${str} MATCHES "Thread model: ")
                string(SUBSTRING ${str} 14 -1 _target)
                break()
            endif ()
        endforeach ()
        if (NOT _thread_model)
            message(FATAL_ERROR "Failed to parse target of C compiler from output \"${out_string}\"")
        endif ()
        set(${GCT_OUT_VAR_THREAD_MODEL} ${_thread_model} PARENT_SCOPE)
    endif ()
endfunction()

function(standardize_arch_for_rust input out_var)
    string(TOLOWER ${input} input)

    set(standard_names "aarch64;aarch64_be;arm;arm64_32;armeb;armebv7r;armv4t;armv5te;armv6;armv6k;armv7;armv7a;armv7k;armv7r;armv7s;asmjs;avr;bpfeb;bpfel;csky;hexagon;i386;i586;i686;loongarch64;m68k;mips;mips64;mips64el;mipsel;mipsisa32r6;mipsisa32r6el;mipsisa64r6;mipsisa64r6el;msp430;nvptx64;powerpc;powerpc64;powerpc64le;riscv32gc;riscv32i;riscv32im;riscv32imac;riscv32imc;riscv64;riscv64gc;riscv64imac;s390x;sparc;sparc64;sparcv9;thumbv4t;thumbv5te;thumbv6m;thumbv7a;thumbv7em;thumbv7m;thumbv7neon;thumbv8m.base;thumbv8m.main;wasm32;wasm64;x86_64;x86_64h")
    if (${input} IN_LIST standard_names)
        set(${out_var} ${input} PARENT_SCOPE)
        return()
    endif ()

    # x86-64
    if (${input} STREQUAL "amd64")
        set(${out_var} "x86_64" PARENT_SCOPE)
        return()
    endif ()

    message(WARNING "Unknown arch name \"${input}\", the arch will be unchanged")
    set(${out_var} ${input} PARENT_SCOPE)
endfunction()

function(get_suggested_cargo_target out_var_target)
    unset(${out_var_target} PARENT_SCOPE)

    #    message("CMAKE_SYSTEM_PROCESSOR = ${CMAKE_SYSTEM_PROCESSOR}")
    standardize_arch_for_rust(${CMAKE_SYSTEM_PROCESSOR} arch_name)
    message("Standardised arch name = ${arch_name}")


    message("CMAKE_C_COMPILER_ID = ${CMAKE_C_COMPILER_ID}")
    # get proper target from llvm-based compiler directly
    if (CMAKE_C_COMPILER_ID STREQUAL "Clang")
        get_clang_target(OUT_VAR_TARGET clang_target)
        message("clang target = ${clang_target}")
        set(${out_var_target} ${clang_target} PARENT_SCOPE)
        return()
    endif ()

    if (WIN32)
        if (${MSVC})
            set(${out_var_target} "${arch_name}-pc-windows-msvc" PARENT_SCOPE)
            return()
        endif ()
        if (${CMAKE_C_COMPILER_ID} STREQUAL "GNU")
            set(${out_var_target} "${arch_name}-pc-windows-gnu" PARENT_SCOPE)
            return()
        endif ()

        # fallback option
        set(${out_var_target} "x86_64-pc-windows-msvc" PARENT_SCOPE)
        message(WARNING "Can not find an accurate suggested target, use fallback value ${out_var_target}")
        return()
    endif ()

    if (LINUX)
        if (${CMAKE_C_COMPILER_ID} STREQUAL "GNU")
            set(${out_var_target} "${arch_name}-unknown-linux-gnu" PARENT_SCOPE)
            return()
        endif ()
        # fallback option
        set(${out_var_target} "x86_64-unknown-linux-gnu" PARENT_SCOPE)
        message(WARNING "Can not find an accurate suggested target, use fallback value ${out_var_target}")
        return()
    endif ()

    if (APPLE)

        # Deduce target by clang in system
        get_clang_target(OUT_VAR_TARGET clang_target CLANG_COMPILER "clang")
        set(${out_var_target} ${clang_target} PARENT_SCOPE)
        return()


        # fallback option
        set(${out_var_target} "x86_64-apple-darwin" PARENT_SCOPE)
        message(WARNING "Can not find an accurate suggested target, use fallback value ${out_var_target}")
        return()
    endif ()

    message(FATAL_ERROR "Can not guess suggested target for rust.")
endfunction()