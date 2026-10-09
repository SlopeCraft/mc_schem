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
#include <cstdint>
#include <filesystem>
#include <mc_schem.hpp>
#include <tuple>
#include <vector>

#include "utils.hpp"

/// Coverage of the block entity (tile entity) wrapper. `test_files/litematica/
/// full-blocks-1.14.4.litematic` is used because it carries well over a
/// thousand tile entities, as non-trivial data. Nothing below depends on how
/// many there are, on where they sit or on what their tags hold.
/// C functions behind every section:
///   mc_schem_create_block_entity,
///   mc_schem_destroy_block_entity (deleter of unique_block_entity),
///   mc_schem_clone_block_entity,
///   mc_schem_block_entity_get_tags, mc_schem_block_entity_get_tags_mut,
///   mc_schem_block_entity_set_tags,
///   mc_schem_region_get_block_entities_count,
///   mc_schem_region_visit_block_entities,
///   mc_schem_region_get_block_entity, mc_schem_region_get_block_entity_mut,
///   mc_schem_region_add_block_entity
int main(int argc, char** argv) {
  using namespace mc_schem;

  // Convention shared with the caller and with ctest: argv[1] is the directory
  // that holds the test fixtures. It is taken as is, the guard below only keeps
  // a missing argument from turning into undefined behaviour.
  MC_SCHEM_CHECK(argc >= 2);
  const std::filesystem::path test_files_dir{argv[1]};

  auto schem = load_schematic_or_abort(
      test_files_dir, "litematica/full-blocks-1.14.4.litematic");
  region& r = first_region_of(schem);
  const region& cr = r;

  std::vector<pos_t> positions;

  // mc_schem_region_get_block_entities_count / mc_schem_region_visit_block_entities
  {
    const size_t count = cr.block_entities_count();
    // At least one is needed for the calls below to run at all
    MC_SCHEM_CHECK(count >= 1);

    cr.visit_block_entities(
        [&positions](int32_t x, int32_t y, int32_t z, const block_entity&) {
          positions.push_back(pos_t{x, y, z});
        });
    MC_SCHEM_CHECK(positions.size() == count);

    // Every position has to be reported exactly once
    std::vector<pos_t> sorted = positions;
    std::sort(sorted.begin(), sorted.end());
    MC_SCHEM_CHECK(std::adjacent_find(sorted.begin(), sorted.end()) ==
                   sorted.end());
  }

  // mc_schem_create_block_entity / mc_schem_block_entity_get_tags /
  // mc_schem_block_entity_get_tags_mut / mc_schem_block_entity_set_tags
  {
    // A block entity made from scratch carries an empty tag map
    auto fresh = block_entity::create();
    MC_SCHEM_CHECK(fresh);

    const nbt_hashmap* fresh_tags = fresh->tags();
    MC_SCHEM_CHECK(fresh_tags not_eq nullptr);
    MC_SCHEM_CHECK(fresh_tags->size() == 0);
    // Both accessors hand out the same map
    MC_SCHEM_CHECK(fresh->tags() == fresh_tags);

    // Take the block entity that carries the most tags, so that the round trip
    // below is as far from vacuous as the fixture allows
    const block_entity* sample = nullptr;
    size_t sample_size = 0;
    for (const pos_t& pos : positions) {
      const block_entity* be = cr.get_block_entity(pos);
      MC_SCHEM_CHECK(be not_eq nullptr);
      const nbt_hashmap* tags = be->tags();
      MC_SCHEM_CHECK(tags not_eq nullptr);
      if (sample == nullptr or tags->size() > sample_size) {
        sample = be;
        sample_size = tags->size();
      }
    }
    MC_SCHEM_CHECK(sample not_eq nullptr);
    // At least one tag is needed for the round trip to check anything
    MC_SCHEM_CHECK(sample_size >= 1);

    // Copying tags in hands the previous value back
    auto previous = fresh->set_tags(*sample->tags());
    MC_SCHEM_CHECK(previous not_eq nullptr);
    MC_SCHEM_CHECK(previous->size() == 0);
    MC_SCHEM_CHECK(fresh->tags()->size() == sample_size);
    // The block entity owns a copy, neither the source nor the map that came
    // back is referenced by it
    MC_SCHEM_CHECK(fresh->tags() not_eq sample->tags());
    MC_SCHEM_CHECK(fresh->tags() not_eq previous.get());

    // And the other way round
    auto restored = fresh->set_tags(*previous);
    MC_SCHEM_CHECK(restored not_eq nullptr);
    MC_SCHEM_CHECK(restored->size() == sample_size);
    MC_SCHEM_CHECK(fresh->tags()->size() == 0);
    MC_SCHEM_CHECK(fresh->tags() not_eq restored.get());
  }

  // mc_schem_region_get_block_entity / mc_schem_region_get_block_entity_mut
  {
    for (const pos_t& pos : positions) {
      const block_entity* cent = cr.get_block_entity(pos);
      block_entity* mut = r.get_block_entity(pos);
      MC_SCHEM_CHECK(cent not_eq nullptr);
      MC_SCHEM_CHECK(mut not_eq nullptr);
      // Both accessors hand out the same object, and the same tag map
      MC_SCHEM_CHECK(cent == mut);
      MC_SCHEM_CHECK(cent->tags() not_eq nullptr);
      MC_SCHEM_CHECK(cent->tags() == mut->tags());
    }

    // A position that holds no block entity gives back nothing
    MC_SCHEM_CHECK(cr.get_block_entity({-1, -1, -1}) == nullptr);
    MC_SCHEM_CHECK(r.get_block_entity({-1, -1, -1}) == nullptr);
  }

  // mc_schem_clone_block_entity / mc_schem_destroy_block_entity
  {
    const pos_t pos = positions.front();
    const block_entity* stored = cr.get_block_entity(pos);
    MC_SCHEM_CHECK(stored not_eq nullptr);
    const size_t stored_size = stored->tags()->size();

    // unique_block_entity runs mc_schem_destroy_block_entity when it goes out
    // of scope
    auto copy = stored->clone();
    MC_SCHEM_CHECK(copy);
    MC_SCHEM_CHECK(copy.get() not_eq stored);
    MC_SCHEM_CHECK(copy->tags() not_eq stored->tags());
    MC_SCHEM_CHECK(copy->tags()->size() == stored_size);
    // Cloning did not swap anything out of the region
    MC_SCHEM_CHECK(stored == cr.get_block_entity(pos));

    // The copy is deep: replacing its tags must leave the stored one alone
    auto empty = nbt_hashmap::create();
    auto old = copy->set_tags(*empty);
    MC_SCHEM_CHECK(old not_eq nullptr);
    MC_SCHEM_CHECK(old->size() == stored_size);
    MC_SCHEM_CHECK(copy->tags()->size() == 0);
    MC_SCHEM_CHECK(cr.get_block_entity(pos)->tags()->size() == stored_size);
  }

  // Walking the whole region must report the same block entities as the
  // dedicated lookups. This runs over every cell of the fixture.
  {
    MC_SCHEM_CHECK(cr.is_dense());
    const pos_t shape = cr.size_xyz();
    for (size_t dim = 0; dim < 3; ++dim) {
      MC_SCHEM_CHECK(shape[dim] >= 1);
    }

    const size_t be_count = cr.block_entities_count();
    uint64_t cells = 0;
    uint64_t cells_with_block_entity = 0;
    r.visit_blocks(
        [&cr, &cells, &cells_with_block_entity](
            pos_t pos, uint16_t, const block&, const block_entity* be) {
          ++cells;
          if (be not_eq nullptr) {
            ++cells_with_block_entity;
          }

          // The walk and the lookups have to agree, whether or not this cell
          // carries a block entity
          MC_SCHEM_CHECK(cr.get_block_entity(pos) == be);

          const auto info = cr.block_info_at(pos);
          MC_SCHEM_CHECK(info.has_value());
          MC_SCHEM_CHECK(std::get<2>(info.value()) == be);
        },
        false);

    // A dense region is walked cell by cell, and a cell can report at most the
    // one block entity that sits on it
    MC_SCHEM_CHECK(cells ==
                   static_cast<uint64_t>(shape[0]) * shape[1] * shape[2]);
    MC_SCHEM_CHECK(cells_with_block_entity <= be_count);
    // Needed so the comparison above is not vacuous
    MC_SCHEM_CHECK(cells_with_block_entity >= 1);
  }

  // mc_schem_region_add_block_entity
  {
    const size_t count = cr.block_entities_count();
    MC_SCHEM_CHECK(count >= 1);

    const pos_t pos = positions.front();
    const block_entity* stored = cr.get_block_entity(pos);
    MC_SCHEM_CHECK(stored not_eq nullptr);

    // Replacing a block entity hands the previous one back and keeps the count
    auto replaced = r.add_block_entity(pos, *stored);
    MC_SCHEM_CHECK(replaced not_eq nullptr);
    MC_SCHEM_CHECK(cr.block_entities_count() == count);
    // What the region holds now is a copy, not the value that was moved out
    MC_SCHEM_CHECK(r.get_block_entity(pos) not_eq replaced.get());

    // Erasing hands it back, erasing the now empty position gives null
    auto removed = r.erase_block_entity(pos);
    MC_SCHEM_CHECK(removed not_eq nullptr);
    MC_SCHEM_CHECK(cr.block_entities_count() == count - 1);
    MC_SCHEM_CHECK(cr.get_block_entity(pos) == nullptr);

    auto nothing = r.erase_block_entity(pos);
    MC_SCHEM_CHECK(nothing == nullptr);
  }

  return 0;
}
