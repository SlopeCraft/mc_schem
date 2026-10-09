//
// Created by Joseph on 2026/10/8.
//
#pragma once

#include <array>
#include <cstddef>
#include <cstdint>
#include <cstdio>
#include <cstdlib>
#include <filesystem>
#include <mc_schem.hpp>
#include <print>
#include <string>
#include <utility>

/// Check `cond`. Unlike `assert`, it is never removed by `NDEBUG`, so the tests
/// keep their meaning in a release build. On failure the failed expression and
/// its location are printed to stderr, then the process aborts.
#define MC_SCHEM_CHECK(cond)                                             \
  if (not(cond)) {                                                       \
    std::println(stderr, "MC_SCHEM_CHECK failed: {}\n  at {}:{}", #cond, \
                 __FILE__, __LINE__);                                    \
    std::abort();                                                        \
  }

/// A block coordinate, or a position relative to a region
using pos_t = std::array<int32_t, 3>;
/// A position in floating point, as carried by an entity
using position_t = std::array<double, 3>;

/// Load a schematic by fixture name, picking the format from its extension.
/// `test_files_dir` is the directory handed over as argv[1] of the test.
inline mc_schem::unique_schematic load_schematic_or_abort(
    const std::filesystem::path& test_files_dir, const std::string& relative) {
  const std::filesystem::path path = test_files_dir / relative;
  auto result = mc_schem::schematic::load_from_file(path.string().c_str());
  if (not result.has_value()) {
    std::println(stderr, "Failed to load {}: {}", path.string(),
                 result.error()->message());
  }
  MC_SCHEM_CHECK(result.has_value());
  // The error type of the expectation holds a unique_ptr, and the standard
  // requires value() & to throw bad_expected_access(as_const(error())), which
  // needs a copy. operator* moves the schematic out instead.
  return *std::move(result);
}

/// The first region of a fixture, as a mutable reference. Fixtures are reached
/// through the schematic API because it is the only way to get a region with
/// non-trivial contents.
inline mc_schem::region& first_region_of(
    const mc_schem::unique_schematic& schem) {
  auto* r = schem->get_region(size_t{0});
  MC_SCHEM_CHECK(r not_eq nullptr);
  return *r;
}

