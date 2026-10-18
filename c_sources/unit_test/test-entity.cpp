/*
 mc_schem is a rust library to generate, load, manipulate and save minecraft
 schematic files. Copyright (C) 2026 ToKiNoBug

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

#include <array>
#include <cmath>
#include <cstdint>
#include <filesystem>
#include <limits>
#include <mc_schem.hpp>
#include <utility>

#include "utils.hpp"

/// Coverage of the entity wrapper. Entities come out of `test02.litematic` and
/// from `entity::create`, which turns a tag map back into one. The fixture is only
/// a source of real entity data: nothing below depends on how many entities it
/// holds, on where they stand or on their tags, so it can be swapped for any
/// other projection that carries entities.
/// C functions behind every section:
///   mc_schem_destroy_entity (deleter of unique_entity),
///   mc_schem_create_entity, mc_schem_create_entity_move,
///   mc_schem_clone_entity, mc_schem_swap_entity,
///   mc_schem_entity_to_nbt,
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

      // That map is the raw nbt of this entity with `Pos` and `id` already
      // parsed out. The section at the end exercises to_nbt() and feeding the
      // result back into entity::create.
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

  // mc_schem_swap_entity
  {
    const entity* stored = cr.get_entity(0);
    MC_SCHEM_CHECK(stored not_eq nullptr);
    auto first = stored->clone();
    auto second = stored->clone();
    MC_SCHEM_CHECK(first);
    MC_SCHEM_CHECK(second);

    // Empty the tags of one copy so that the swap is observable
    auto empty = nbt_hashmap::create();
    MC_SCHEM_CHECK(second->set_tags(*empty) not_eq nullptr);
    const size_t first_size = first->tags()->size();
    MC_SCHEM_CHECK(second->tags()->size() == 0);

    first->swap(*second);
    MC_SCHEM_CHECK(first->tags()->size() == 0);
    MC_SCHEM_CHECK(second->tags()->size() == first_size);
    // The entity the copies were made from is left alone
    MC_SCHEM_CHECK(stored->tags()->size() == first_size);
  }

  // mc_schem_entity_to_nbt, mc_schem_create_entity,
  // mc_schem_create_entity_move: round trip of an entity through nbt. The tag
  // map of an entity carries its nbt with `Pos` and `id` parsed out, to_nbt()
  // hands out a map that holds them again. What the map contains is up to the
  // Rust side; what matters here is that every call returns a usable handle and
  // that none of them owns a map twice.
  {
    const entity* stored = cr.get_entity(0);
    MC_SCHEM_CHECK(stored not_eq nullptr);

    auto full = stored->to_nbt();
    MC_SCHEM_CHECK(full);

    // Both overloads are called; the moving one consumes the map it is given
    [[maybe_unused]] auto parsed = entity::create(*full);
    auto again = stored->to_nbt();
    MC_SCHEM_CHECK(again);
    [[maybe_unused]] auto moved = entity::create(std::move(again));
    MC_SCHEM_CHECK(again == nullptr);

    // A map without a Pos tag makes the call fail: neither the entity nor the
    // error it reports may be leaked (asan checks that), so the result is just
    // destroyed here
    auto empty = nbt_hashmap::create();
    MC_SCHEM_CHECK(empty);
    [[maybe_unused]] auto rejected = entity::create(*empty);
  }

  return 0;
}
