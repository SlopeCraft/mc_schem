/*
 mc_schem is a rust library to generate, load, manipulate and save minecraft
 schematic files. Copyright (C) 2024  joseph

 This program is free software: you can redistribute it and/or modify it under
 the terms of the GNU General Public License as published by the Free Software
 Foundation, either version 3 of the License, or (at your option) any later
 version.

 This program is distributed in the hope that it will be useful, but WITHOUT ANY
 WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A
 PARTICULAR PURPOSE.  See the GNU General Public License for more details.

 You should have received a copy of the GNU General Public License
 along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

#include <algorithm>
#include <array>
#include <cmath>
#include <cstdint>
#include <cstdlib>
#include <exception>
#include <filesystem>
#include <limits>
#include <mc_schem.hpp>
#include <print>
#include <string>
#include <string_view>
#include <utility>
#include <vector>

#include "utils.hpp"

namespace {

using block_pos_t = std::array<int32_t, 3>;
using position_t = std::array<double, 3>;

/// Load a schematic by fixture name, picking the format from its extension.
/// `test_files_dir` is the directory handed over as argv[1], see main().
mc_schem::unique_schematic load_schematic_or_abort(
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

/// The first region of a fixture, as a mutable reference. This one call into
/// the schematic API is only here to obtain entities to work with, because
/// there is no way to create an entity from C++.
mc_schem::region& first_region_of(const mc_schem::unique_schematic& schem) {
  auto* r = schem->get_region(size_t{0});
  MC_SCHEM_CHECK(r not_eq nullptr);
  return *r;
}

/// Dump the tags of an entity, reporting the failure instead of letting an
/// uncaught exception terminate the test.
std::vector<uint8_t> dump_tags_or_abort(const mc_schem::nbt_hashmap& tags) {
  try {
    return tags.dump();
  } catch (const std::exception& e) {
    std::println(stderr, "Failed to dump entity tags: {}", e.what());
    std::abort();
  }
}

/// True if `needle` shows up in `data` as raw bytes.
bool contains_ascii(const std::vector<uint8_t>& data, std::string_view needle) {
  const auto found =
      std::search(data.begin(), data.end(), needle.begin(), needle.end(),
                  [](uint8_t lhs, char rhs) {
                    return lhs == static_cast<uint8_t>(rhs);
                  });
  return found not_eq data.end();
}

}  // namespace

/// Coverage of the entity wrapper. There is no exported way to create an entity
/// from C++, so all of them come out of `test02.litematic`. The fixture is only
/// a source of real entity data: nothing below depends on how many entities it
/// holds, on where they stand or on their tags, so it can be swapped for any
/// other projection that carries entities.
/// C functions behind every section:
///   mc_schem_destroy_entity (deleter of unique_entity),
///   mc_schem_clone_entity,
///   mc_schem_entity_get_position,
///   mc_schem_entity_get_tags, mc_schem_entity_get_tags_mut,
///   mc_schem_entity_set_tags
int main(int argc, char** argv) {
  using namespace mc_schem;

  // Convention shared with the caller and with ctest: argv[1] is the directory
  // that holds the test fixtures. It is taken as is, the guard below only keeps
  // a missing argument from turning into undefined behaviour.
  MC_SCHEM_CHECK(argc >= 2);
  const std::filesystem::path test_files_dir{argv[1]};

  auto schem =
      load_schematic_or_abort(test_files_dir, "litematica/test02.litematic");
  region& r = first_region_of(schem);
  const region& cr = r;

  // At least one entity is needed for the calls below to run at all
  const size_t count = cr.entities_count();
  MC_SCHEM_CHECK(count >= 1);

  // mc_schem_entity_get_position
  {
    constexpr double min_i32 =
        static_cast<double>(std::numeric_limits<int32_t>::min());
    constexpr double max_i32 =
        static_cast<double>(std::numeric_limits<int32_t>::max());

    for (size_t i = 0; i < count; ++i) {
      const entity* ent = cr.get_entity(i);
      MC_SCHEM_CHECK(ent not_eq nullptr);

      const auto [block_pos, position] = ent->position();
      for (size_t dim = 0; dim < 3; ++dim) {
        const double value = position[dim];
        // The integer position is the double one truncated towards zero, which
        // is what `as i32` does on the Rust side. The range guard keeps the
        // cast well defined.
        MC_SCHEM_CHECK(std::isfinite(value));
        MC_SCHEM_CHECK(value >= min_i32 and value <= max_i32);
        MC_SCHEM_CHECK(static_cast<int32_t>(value) == block_pos[dim]);
      }
    }

    // Out of range lookups return null instead of an entity
    MC_SCHEM_CHECK(cr.get_entity(count) == nullptr);
    MC_SCHEM_CHECK(r.get_entity(count) == nullptr);
  }

  // mc_schem_entity_get_tags / mc_schem_entity_get_tags_mut
  {
    for (size_t i = 0; i < count; ++i) {
      const entity* cent = cr.get_entity(i);
      entity* ent = r.get_entity(i);
      MC_SCHEM_CHECK(cent not_eq nullptr);
      MC_SCHEM_CHECK(ent not_eq nullptr);

      const nbt_hashmap* tags = cent->tags();
      MC_SCHEM_CHECK(tags not_eq nullptr);
      MC_SCHEM_CHECK(tags->size() >= 1);

      // The mutable accessor has to hand out the very same map
      MC_SCHEM_CHECK(ent->tags() == tags);

      // And that map is really the nbt of this entity. An entity without a
      // `Pos` tag is rejected while loading, so the key is present for every
      // entity of every fixture.
      MC_SCHEM_CHECK(contains_ascii(dump_tags_or_abort(*tags), "Pos"));
    }
  }

  // mc_schem_entity_set_tags: deep copy in, previous value back
  {
    entity* ent = r.get_entity(0);
    MC_SCHEM_CHECK(ent not_eq nullptr);

    const size_t original_size = ent->tags()->size();
    MC_SCHEM_CHECK(original_size >= 1);

    auto replacement = nbt_hashmap::create();
    MC_SCHEM_CHECK(replacement);
    MC_SCHEM_CHECK(replacement->size() == 0);

    // The entity takes a copy of the new tags and hands the old ones over
    auto previous = ent->set_tags(*replacement);
    MC_SCHEM_CHECK(previous not_eq nullptr);
    MC_SCHEM_CHECK(previous->size() == original_size);
    MC_SCHEM_CHECK(ent->tags()->size() == 0);
    // The entity owns a copy: neither the map that was passed in nor the one
    // that came back is referenced by it any more
    MC_SCHEM_CHECK(ent->tags() not_eq replacement.get());
    MC_SCHEM_CHECK(ent->tags() not_eq previous.get());

    // Putting them back hands over the empty map that was just installed
    auto discarded = ent->set_tags(*previous);
    MC_SCHEM_CHECK(discarded not_eq nullptr);
    MC_SCHEM_CHECK(discarded->size() == 0);
    MC_SCHEM_CHECK(ent->tags()->size() == original_size);
    MC_SCHEM_CHECK(ent->tags() not_eq previous.get());
    MC_SCHEM_CHECK(ent->tags() not_eq discarded.get());

    // The restored tags still hold the data of that entity
    MC_SCHEM_CHECK(contains_ascii(dump_tags_or_abort(*ent->tags()), "Pos"));
  }

  // mc_schem_clone_entity / mc_schem_destroy_entity
  {
    const entity* src = cr.get_entity(0);
    MC_SCHEM_CHECK(src not_eq nullptr);

    auto copy = src->clone();
    MC_SCHEM_CHECK(copy);

    MC_SCHEM_CHECK(copy->position() == src->position());
    MC_SCHEM_CHECK(copy->tags() not_eq src->tags());
    MC_SCHEM_CHECK(copy->tags()->size() == src->tags()->size());

    // The copy is deep: replacing the tags of the copy leaves the original,
    // which still belongs to the region, untouched
    const size_t src_size = src->tags()->size();
    auto empty = nbt_hashmap::create();
    auto old = copy->set_tags(*empty);
    MC_SCHEM_CHECK(old not_eq nullptr);
    MC_SCHEM_CHECK(old->size() == src_size);
    MC_SCHEM_CHECK(copy->tags()->size() == 0);
    MC_SCHEM_CHECK(src->tags()->size() == src_size);
  }

  return 0;
}
