//
// Created by Joseph on 2026/10/8.
//
#pragma once

#include <cstdio>
#include <cstdlib>
#include <print>

#define MC_SCHEM_CHECK(cond)                                             \
  if (not(cond)) {                                                       \
    std::println(stderr, "MC_SCHEM_CHECK failed: {}\n  at {}:{}", #cond, \
                 __FILE__, __LINE__);                                    \
    std::abort();                                                        \
  }
