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

#include <array>
#include <cstdint>
#include <filesystem>
#include <fstream>
#include <mc_schem.hpp>
#include <optional>
#include <print>
#include <sstream>
#include <string>
#include <utility>
#include <vector>

#include "utils.hpp"

using namespace mc_schem;

namespace {

/// `pos - offset`
pos_t relative_to(const pos_t& pos, const pos_t& offset) {
  return pos_t{pos[0] - offset[0], pos[1] - offset[1], pos[2] - offset[2]};
}

/// The aggregate values have to agree with what the regions report
void check_aggregates(const schematic& schem) {
  const pos_t shape = schem.shape();
  uint64_t expected_volume = 1;
  for (size_t dim = 0; dim < 3; ++dim) {
    if (schem.regions_count() > 0) {
      MC_SCHEM_CHECK(shape[dim] >= 1);
    }
    expected_volume *= static_cast<uint64_t>(shape[dim]);
  }
  MC_SCHEM_CHECK(schem.volume() == expected_volume);

  uint64_t cells_of_all_regions = 0;
  uint64_t solid_blocks = 0;
  for (size_t i = 0; i < schem.regions_count(); ++i) {
    const region* reg = schem.get_region(i);
    MC_SCHEM_CHECK(reg not_eq nullptr);
    const pos_t rshape = reg->size_xyz();
    for (size_t dim = 0; dim < 3; ++dim) {
      MC_SCHEM_CHECK(rshape[dim] >= 1);
    }
    cells_of_all_regions +=
        static_cast<uint64_t>(rshape[0]) * rshape[1] * rshape[2];
    solid_blocks += reg->total_blocks(false);
  }

  // total_blocks walks every cell of every region, so the count includes the
  // cells of overlapping regions twice
  MC_SCHEM_CHECK(schem.total_blocks(true) == cells_of_all_regions);
  MC_SCHEM_CHECK(schem.total_blocks(false) == solid_blocks);
}

/// For every cell of every region, the "first hit" lookups of the schematic
/// have to agree with the region that cell belongs to
void check_first_hit(const schematic& schem) {
  const size_t region_count = schem.regions_count();
  for (size_t i = 0; i < region_count; ++i) {
    const region* reg = schem.get_region(i);
    MC_SCHEM_CHECK(reg not_eq nullptr);
    const pos_t offset = reg->offset();
    const pos_t shape = reg->size_xyz();

    for (int32_t x = 0; x < shape[0]; ++x) {
      for (int32_t y = 0; y < shape[1]; ++y) {
        for (int32_t z = 0; z < shape[2]; ++z) {
          const pos_t pos{x + offset[0], y + offset[1], z + offset[2]};

          // The cell belongs to region `i`, so the first hit is that region or
          // one before it when regions overlap
          const auto hit = schem.first_region_index_at(pos);
          MC_SCHEM_CHECK(hit.has_value());
          MC_SCHEM_CHECK(hit.value() <= i);

          const region* first = schem.get_region(hit.value());
          MC_SCHEM_CHECK(first not_eq nullptr);
          const auto expected =
              first->block_index_at(relative_to(pos, first->offset()));
          MC_SCHEM_CHECK(expected.has_value());

          const auto index = schem.first_block_index_at(pos);
          MC_SCHEM_CHECK(index.has_value());
          MC_SCHEM_CHECK(index.value() == expected.value());

          const block* blk = schem.first_block_at(pos);
          MC_SCHEM_CHECK(blk not_eq nullptr);
          MC_SCHEM_CHECK(blk == first->palette(expected.value()));
        }
      }
    }
  }

  // A block entity and a pending tick that a region holds must be reachable
  // through the schematic as well
  for (size_t i = 0; i < region_count; ++i) {
    const region* reg = schem.get_region(i);
    MC_SCHEM_CHECK(reg not_eq nullptr);
    const pos_t offset = reg->offset();
    const pos_t shape = reg->size_xyz();

    reg->visit_block_entities(
        [&](int32_t x, int32_t y, int32_t z, const block_entity&) {
          const pos_t pos{x + offset[0], y + offset[1], z + offset[2]};
          MC_SCHEM_CHECK(schem.first_block_entity_at(pos) not_eq nullptr);
        });

    for (int32_t x = 0; x < shape[0]; ++x) {
      for (int32_t y = 0; y < shape[1]; ++y) {
        for (int32_t z = 0; z < shape[2]; ++z) {
          const pos_t rel{x, y, z};
          if (reg->pending_ticks_count_at(rel) == 0) {
            continue;
          }
          MC_SCHEM_CHECK(schem
                             .first_pending_ticks_at(pos_t{
                                 x + offset[0], y + offset[1], z + offset[2]})
                             .size() >= 1);
        }
      }
    }
  }
}

/// `merged` is expected to reproduce, cell by cell, what `schem` reports, with
/// `background` wherever no region covers the cell
void check_merged(const schematic& schem, const region& merged,
                  const block& background) {
  MC_SCHEM_CHECK((merged.offset() == pos_t{0, 0, 0}));
  const pos_t shape = schem.shape();
  MC_SCHEM_CHECK((merged.size_xyz() == shape));

  // `to_single_region` walks the cells of the merged region from the origin on,
  // so a cell is taken from a region only when its absolute position falls
  // inside that box. Count those cells, and the block entities that have to be
  // carried over, up front instead of assuming anything.
  const uint64_t box_cells =
      static_cast<uint64_t>(shape[0]) * shape[1] * shape[2];
  const auto inside_box = [&shape](const pos_t& pos) {
    return pos[0] >= 0 and pos[0] < shape[0] and pos[1] >= 0 and
           pos[1] < shape[1] and pos[2] >= 0 and pos[2] < shape[2];
  };

  std::vector<uint8_t> taken(box_cells, 0);
  uint64_t expected_block_entities = 0;
  for (size_t i = 0; i < schem.regions_count(); ++i) {
    const region* reg = schem.get_region(i);
    MC_SCHEM_CHECK(reg not_eq nullptr);
    const pos_t offset = reg->offset();
    const pos_t rshape = reg->size_xyz();
    for (int32_t x = 0; x < rshape[0]; ++x) {
      for (int32_t y = 0; y < rshape[1]; ++y) {
        for (int32_t z = 0; z < rshape[2]; ++z) {
          const pos_t pos{x + offset[0], y + offset[1], z + offset[2]};
          if (not inside_box(pos)) {
            continue;
          }
          taken[(static_cast<uint64_t>(pos[0]) * shape[1] + pos[1]) * shape[2] +
                pos[2]] = 1;
        }
      }
    }

    // A block entity is carried over when its own region is the first hit of
    // the cell it sits on
    reg->visit_block_entities(
        [&](int32_t x, int32_t y, int32_t z, const block_entity&) {
          const pos_t pos{x + offset[0], y + offset[1], z + offset[2]};
          if (not inside_box(pos)) {
            return;
          }
          const auto hit = schem.first_region_index_at(pos);
          if (hit.has_value() and hit.value() == i) {
            ++expected_block_entities;
          }
        });
  }
  uint64_t expected_covered = 0;
  for (const uint8_t flag : taken) {
    expected_covered += flag;
  }

  const std::string background_id = background.full_id();
  uint64_t cells = 0;
  uint64_t covered = 0;
  merged.visit_blocks(
      [&](pos_t pos, uint16_t idx, const block& blk, const block_entity* be) {
        ++cells;
        MC_SCHEM_CHECK(merged.palette(idx) == &blk);

        // The first hit decides the block, the ticks and the block entity
        const block* first_blk = schem.first_block_at(pos);
        if (first_blk == nullptr) {
          MC_SCHEM_CHECK(blk.full_id() == background_id);
          MC_SCHEM_CHECK(be == nullptr);
          return;
        }
        ++covered;
        MC_SCHEM_CHECK(blk.full_id() == first_blk->full_id());

        const auto hit = schem.first_region_index_at(pos);
        MC_SCHEM_CHECK(hit.has_value());
        const region* first = schem.get_region(hit.value());
        MC_SCHEM_CHECK(first not_eq nullptr);
        const pos_t rel = relative_to(pos, first->offset());
        MC_SCHEM_CHECK(merged.pending_ticks_count_at(pos) ==
                       first->pending_ticks_count_at(rel));
        if (be not_eq nullptr) {
          // The merged region holds a copy of the block entity of the first
          // hit, so only its presence can be compared, never the address
          MC_SCHEM_CHECK(first->get_block_entity(rel) not_eq nullptr);
        }
      },
      false);

  MC_SCHEM_CHECK(cells == box_cells);
  MC_SCHEM_CHECK(covered == expected_covered);
  MC_SCHEM_CHECK(merged.block_entities_count() == expected_block_entities);
  MC_SCHEM_CHECK(merged.total_blocks(true) == cells);
}

/// Load a fixture by its path, reporting the error before giving up
unique_schematic load_or_abort(const std::string& path) {
  auto result = schematic::load_from_file(path.c_str());
  if (not result.has_value()) {
    std::println(stderr, "Failed to load {}: {}", path,
                 result.error()->message());
  }
  MC_SCHEM_CHECK(result.has_value());
  return *std::move(result);
}

/// The reader based loaders take uncompressed nbt, while every fixture is
/// gzipped, so feeding a file to them has to fail with an error
void check_reader_rejects_gzip(const std::string& path) {
  std::ifstream stream{path, std::ios::binary};
  MC_SCHEM_CHECK(stream.is_open());

  auto litematica =
      schematic::load_litematica(stream, litematica_load_option{});
  MC_SCHEM_CHECK(not litematica.has_value());
  MC_SCHEM_CHECK(not litematica.error()->message().empty());

  stream.clear();
  stream.seekg(0);
  auto vanilla = schematic::load_vanilla_structure(
      stream, vanilla_structure_load_option{});
  MC_SCHEM_CHECK(not vanilla.has_value());
  MC_SCHEM_CHECK(not vanilla.error()->message().empty());

  stream.clear();
  stream.seekg(0);
  auto we13 = schematic::load_world_edit13(stream, world_edit13_load_option{});
  MC_SCHEM_CHECK(not we13.has_value());
  MC_SCHEM_CHECK(not we13.error()->message().empty());

  stream.clear();
  stream.seekg(0);
  auto we12 = schematic::load_world_edit12(stream, world_edit12_load_option{});
  MC_SCHEM_CHECK(not we12.has_value());
  MC_SCHEM_CHECK(not we12.error()->message().empty());
}

}  // namespace

/// Coverage of the schematic wrapper. `full-blocks-26.2.litematic` provides a
/// single region holding a huge number of block types, `multi-region01.
/// litematic` provides 22 regions with different offsets, and one fixture of
/// each other supported format covers the parsers. Nothing below depends on
/// how many regions or blocks a fixture happens to hold.
/// C functions behind every section:
///   mc_schem_create_schematic, mc_schem_destroy_schematic (deleter),
///   mc_schem_clone_schematic,
///   mc_schem_schematic_get_metadata, mc_schem_schematic_get_metadata_mut,
///   mc_schem_schematic_set_metadata,
///   mc_schem_schematic_get_regions_count, mc_schem_schematic_get_region,
///   mc_schem_schematic_get_region_mut, mc_schem_schematic_remove_region,
///   mc_schem_schematic_clear_all_regions, mc_schem_schematic_insert_region,
///   mc_schem_schematic_insert_region_move,
///   mc_schem_schematic_get_first_region_index_at,
///   mc_schem_schematic_get_first_block_index_at,
///   mc_schem_schematic_get_first_block_at,
///   mc_schem_schematic_get_first_block_entity_at,
///   mc_schem_schematic_get_first_pending_ticks_count_at,
///   mc_schem_schematic_get_first_pending_ticks_at,
///   mc_schem_schematic_get_shape, mc_schem_schematic_get_volume,
///   mc_schem_schematic_get_total_blocks,
///   mc_schem_schematic_to_single_region, mc_schem_schematic_merge_regions,
///   mc_schem_schematic_load_litematica_from_reader,
///   mc_schem_schematic_load_vanilla_structure_from_reader,
///   mc_schem_schematic_load_world_edit13_from_reader,
///   mc_schem_schematic_load_world_edit12_from_reader,
///   mc_schem_schematic_load_from_file,
///   mc_schem_schematic_save_litematica_to_writer,
///   mc_schem_schematic_save_world_edit13_to_writer,
///   mc_schem_schematic_save_vanilla_structure_to_writer,
///   mc_schem_schematic_save_to_file
int main(int argc, char** argv) {
  MC_SCHEM_CHECK(argc >= 2);
  const std::filesystem::path test_files_dir{argv[1]};

  auto air = block::from_common(common_block::air);

  // mc_schem_create_schematic and the lookups on a schematic without regions
  {
    auto schem = schematic::create();
    MC_SCHEM_CHECK(schem);
    const schematic& cs = *schem;

    MC_SCHEM_CHECK(cs.regions_count() == 0);
    MC_SCHEM_CHECK((cs.shape() == pos_t{0, 0, 0}));
    MC_SCHEM_CHECK(cs.volume() == 0);
    MC_SCHEM_CHECK(cs.total_blocks(false) == 0);
    MC_SCHEM_CHECK(cs.total_blocks(true) == 0);
    MC_SCHEM_CHECK(cs.get_region(0) == nullptr);
    MC_SCHEM_CHECK(cs.metadata() not_eq nullptr);
    MC_SCHEM_CHECK(cs.metadata()->mc_data_version() >= 1);

    MC_SCHEM_CHECK(not cs.first_region_index_at({0, 0, 0}).has_value());
    MC_SCHEM_CHECK(not cs.first_block_index_at({0, 0, 0}).has_value());
    MC_SCHEM_CHECK(cs.first_block_at({0, 0, 0}) == nullptr);
    MC_SCHEM_CHECK(cs.first_block_entity_at({0, 0, 0}) == nullptr);
    MC_SCHEM_CHECK(cs.first_pending_ticks_at({0, 0, 0}).empty());

    // Removing from an empty schematic is refused instead of crashing
    MC_SCHEM_CHECK(schem->remove_region(0) == nullptr);
    // An index past the end is refused as well
    MC_SCHEM_CHECK(schem->insert_region(nullptr, 1) == nullptr);

    check_aggregates(cs);

    // Merging an empty schematic still produces a single region
    auto merged = cs.to_single_region(*air);
    MC_SCHEM_CHECK(merged);
    MC_SCHEM_CHECK((merged->size_xyz() == pos_t{0, 0, 0}));
  }

  // Inserting, removing and clearing regions, plus the meta data accessors
  {
    auto schem = schematic::create();
    MC_SCHEM_CHECK(schem);

    auto first = region::create(2, 2, 2);
    MC_SCHEM_CHECK(first);
    first->set_name("first");
    first->set_offset({10, 0, 0});
    auto stone = block::create();
    MC_SCHEM_CHECK(stone->reset("minecraft:stone").has_value());
    first->set_block({0, 0, 0}, *stone);

    auto second = region::create(3, 1, 1);
    MC_SCHEM_CHECK(second);
    second->set_name("second");
    second->set_offset({0, 5, 0});

    // appending deep copies: the schematic must not alias the source region
    region* appended = schem->append_region(first.get());
    MC_SCHEM_CHECK(appended not_eq nullptr);
    MC_SCHEM_CHECK(appended not_eq first.get());
    MC_SCHEM_CHECK(schem->regions_count() == 1);
    MC_SCHEM_CHECK(appended->name() == "first");
    MC_SCHEM_CHECK((appended->offset() == pos_t{10, 0, 0}));
    MC_SCHEM_CHECK(appended->palette(1)->full_id() == "minecraft:stone");

    // changing the source afterwards must not be visible in the schematic
    first->set_name("changed");
    first->set_offset({0, 0, 0});
    MC_SCHEM_CHECK(appended->name() == "first");
    MC_SCHEM_CHECK((appended->offset() == pos_t{10, 0, 0}));

    MC_SCHEM_CHECK(schem->append_region(second.get()) not_eq nullptr);
    MC_SCHEM_CHECK(schem->regions_count() == 2);
    MC_SCHEM_CHECK(schem->get_region(2) == nullptr);
    MC_SCHEM_CHECK(schem->get_region(1)->name() == "second");

    // an insert at the front shifts the others
    auto third = region::create(1, 1, 1);
    MC_SCHEM_CHECK(third);
    third->set_name("third");
    MC_SCHEM_CHECK(schem->insert_region(third.get(), 0) not_eq nullptr);
    MC_SCHEM_CHECK(schem->regions_count() == 3);
    MC_SCHEM_CHECK(schem->get_region(0)->name() == "third");
    MC_SCHEM_CHECK(schem->get_region(1)->name() == "first");
    MC_SCHEM_CHECK(schem->get_region(2)->name() == "second");
    // an index past the end is refused
    MC_SCHEM_CHECK(schem->insert_region(third.get(), 4) == nullptr);
    MC_SCHEM_CHECK(schem->regions_count() == 3);

    // removing hands the region over
    auto removed = schem->remove_region(0);
    MC_SCHEM_CHECK(removed);
    MC_SCHEM_CHECK(removed->name() == "third");
    MC_SCHEM_CHECK(schem->regions_count() == 2);
    MC_SCHEM_CHECK(schem->get_region(0)->name() == "first");

    check_aggregates(*schem);
    check_first_hit(*schem);
    auto merged = schem->to_single_region(*air);
    MC_SCHEM_CHECK(merged);
    check_merged(*schem, *merged, *air);

    // mc_schem_clone_schematic
    auto copy = schem->clone();
    MC_SCHEM_CHECK(copy);
    MC_SCHEM_CHECK(copy->regions_count() == schem->regions_count());
    MC_SCHEM_CHECK((copy->shape() == schem->shape()));
    MC_SCHEM_CHECK(copy->volume() == schem->volume());
    MC_SCHEM_CHECK(copy->total_blocks(true) == schem->total_blocks(true));
    copy->get_region(0)->set_name("renamed");
    MC_SCHEM_CHECK(schem->get_region(0)->name() == "first");

    // the meta data accessors: setting deep copies and hands the old one back
    auto fresh_meta = metadata_ir::create(111);
    MC_SCHEM_CHECK(fresh_meta.has_value());
    auto other_meta = metadata_ir::create(222);
    MC_SCHEM_CHECK(other_meta.has_value());

    const int32_t original_version = schem->metadata()->mc_data_version();
    auto previous = schem->set_metadata(**other_meta);
    MC_SCHEM_CHECK(previous not_eq nullptr);
    MC_SCHEM_CHECK(previous->mc_data_version() == original_version);
    MC_SCHEM_CHECK(schem->metadata()->mc_data_version() == 222);
    // the schematic holds a copy, not the meta data that was handed in
    MC_SCHEM_CHECK(schem->metadata() not_eq other_meta->get());
    // and the mutable accessor hands out the very same object
    MC_SCHEM_CHECK(schem->metadata() == std::as_const(*schem).metadata());

    MC_SCHEM_CHECK(schem->set_metadata(**fresh_meta) not_eq nullptr);
    MC_SCHEM_CHECK(schem->metadata()->mc_data_version() == 111);

    schem->clear_all_regions();
    MC_SCHEM_CHECK(schem->regions_count() == 0);
    MC_SCHEM_CHECK((schem->shape() == pos_t{0, 0, 0}));
    check_aggregates(*schem);
  }

  // Moving a region in, the round trip through remove_region, and the contract
  // that the handle is consumed even when the index is refused
  {
    auto schem = schematic::create();
    MC_SCHEM_CHECK(schem);

    auto first = region::create(1, 1, 1);
    MC_SCHEM_CHECK(first);
    first->set_name("first");
    MC_SCHEM_CHECK(schem->insert_region(std::move(first), 0) not_eq nullptr);
    MC_SCHEM_CHECK(first == nullptr);  // the handle went into the schematic
    MC_SCHEM_CHECK(schem->regions_count() == 1);
    MC_SCHEM_CHECK(schem->get_region(0)->name() == "first");

    // Appending by move puts the region at the end
    auto second = region::create(2, 2, 2);
    MC_SCHEM_CHECK(second);
    second->set_name("second");
    MC_SCHEM_CHECK(schem->append_region(std::move(second)) not_eq nullptr);
    MC_SCHEM_CHECK(second == nullptr);
    MC_SCHEM_CHECK(schem->regions_count() == 2);
    MC_SCHEM_CHECK(schem->get_region(1)->name() == "second");

    // Taking a region out and moving it back in keeps its contents, and hands
    // out a reference into the schematic
    auto taken = schem->remove_region(0);
    MC_SCHEM_CHECK(taken);
    MC_SCHEM_CHECK(taken->name() == "first");
    MC_SCHEM_CHECK(schem->regions_count() == 1);
    region* moved_back = schem->insert_region(std::move(taken), 1);
    MC_SCHEM_CHECK(moved_back not_eq nullptr);
    MC_SCHEM_CHECK(taken == nullptr);
    MC_SCHEM_CHECK(schem->regions_count() == 2);
    MC_SCHEM_CHECK(moved_back == schem->get_region(1));
    MC_SCHEM_CHECK(moved_back->name() == "first");
    MC_SCHEM_CHECK((moved_back->size_xyz() == pos_t{1, 1, 1}));

    // An index past the end is refused, and the region is consumed anyway
    auto third = region::create(3, 3, 3);
    MC_SCHEM_CHECK(third);
    MC_SCHEM_CHECK(schem->insert_region(std::move(third), 3) == nullptr);
    MC_SCHEM_CHECK(third == nullptr);
    MC_SCHEM_CHECK(schem->regions_count() == 2);
  }

  // mc_schem_swap_schematic
  {
    auto with_region = schematic::create();
    MC_SCHEM_CHECK(with_region);
    auto empty = schematic::create();
    MC_SCHEM_CHECK(empty);

    auto reg = region::create(2, 3, 4);
    MC_SCHEM_CHECK(reg);
    MC_SCHEM_CHECK(with_region->append_region(reg.get()) not_eq nullptr);
    MC_SCHEM_CHECK(with_region->regions_count() == 1);
    MC_SCHEM_CHECK(empty->regions_count() == 0);

    with_region->swap(*empty);
    MC_SCHEM_CHECK(with_region->regions_count() == 0);
    MC_SCHEM_CHECK(empty->regions_count() == 1);
    MC_SCHEM_CHECK((empty->shape() == pos_t{2, 3, 4}));
    MC_SCHEM_CHECK((with_region->shape() == pos_t{0, 0, 0}));
  }

  // Every supported format: parsing, the aggregates and the first hit lookups
  {
    const char* const fixtures[] = {
        "litematica/full-blocks-26.2.litematic",
        "litematica/multi-region01.litematic",
        // test03 carries block entities and pending ticks
        "litematica/test03.litematic",
        "schem/test01.schem",
        "schematic/full-blocks-1.12.2.schematic",
        "vanilla_structure/test01.nbt",
    };

    for (const char* const fixture : fixtures) {
      const std::string path = (test_files_dir / fixture).string();
      auto schem = load_or_abort(path);
      const schematic& cs = *schem;

      MC_SCHEM_CHECK(cs.regions_count() >= 1);
      MC_SCHEM_CHECK(cs.metadata() not_eq nullptr);
      check_aggregates(cs);
      check_first_hit(cs);

      for (size_t i = 0; i < cs.regions_count(); ++i) {
        const region* reg = cs.get_region(i);
        MC_SCHEM_CHECK(reg not_eq nullptr);
        MC_SCHEM_CHECK(reg->palette_size() >= 1);
        MC_SCHEM_CHECK(reg->palette(0) not_eq nullptr);
      }

      // The reader based loaders take uncompressed nbt, while the fixtures are
      // gzipped, so they have to report an error instead of parsing
      check_reader_rejects_gzip(path);
    }
  }

  // Merging is only exercised on the multi region fixture. It is the case worth
  // checking (many regions with their own offsets, block entities, and cells
  // covered by more than one region), and both `to_single_region` and the
  // writers are far too slow on the huge palette of the other fixture.
  {
    auto schem = load_or_abort(
        (test_files_dir / "litematica/multi-region01.litematic").string());
    const schematic& cs = *schem;
    const size_t regions_before = cs.regions_count();
    const pos_t shape_before = cs.shape();

    auto merged = cs.to_single_region(*air);
    MC_SCHEM_CHECK(merged);
    check_merged(cs, *merged, *air);

    // Merging in place has to produce the very same region, and since it
    // happens on a clone, the original has to stay untouched
    auto clone = cs.clone();
    MC_SCHEM_CHECK(clone);
    clone->merge_regions(*air);
    MC_SCHEM_CHECK(clone->regions_count() == 1);
    const region* merged_in_place = clone->get_region(0);
    MC_SCHEM_CHECK(merged_in_place not_eq nullptr);
    MC_SCHEM_CHECK((merged_in_place->size_xyz() == merged->size_xyz()));
    MC_SCHEM_CHECK(merged_in_place->palette_size() == merged->palette_size());
    MC_SCHEM_CHECK(merged_in_place->total_blocks(false) ==
                   merged->total_blocks(false));
    MC_SCHEM_CHECK(merged_in_place->total_blocks(true) ==
                   merged->total_blocks(true));
    MC_SCHEM_CHECK(merged_in_place->block_entities_count() ==
                   merged->block_entities_count());

    MC_SCHEM_CHECK(cs.regions_count() == regions_before);
    MC_SCHEM_CHECK((cs.shape() == shape_before));
  }

  // Saving: to a stream, and to files that are parsed back. A small schematic
  // is used on purpose: `full_palette` inside the writers is quadratic in the
  // number of palette entries, and gzipping a multi megabyte nbt at level 9 is
  // extremely slow in a debug build.
  {
    auto schem =
        load_or_abort((test_files_dir / "schem/test01.schem").string());

    // All three save functions write gzipped nbt
    const auto starts_with_gzip_magic = [](const std::string& bytes) {
      return bytes.size() >= 2 and static_cast<uint8_t>(bytes[0]) == 0x1f and
             static_cast<uint8_t>(bytes[1]) == 0x8b;
    };

    {
      std::ostringstream out;
      MC_SCHEM_CHECK(
          schem->save_litematica(out, litematica_save_option{}).has_value());
      MC_SCHEM_CHECK(starts_with_gzip_magic(out.str()));
    }
    {
      std::ostringstream out;
      MC_SCHEM_CHECK(schem->save_world_edit13(out, world_edit13_save_option{})
                         .has_value());
      MC_SCHEM_CHECK(starts_with_gzip_magic(out.str()));
    }
    {
      std::ostringstream out;
      MC_SCHEM_CHECK(
          schem->save_vanilla_structure(out, vanilla_structure_save_option{})
              .has_value());
      MC_SCHEM_CHECK(starts_with_gzip_magic(out.str()));
    }

    // A file format the library does not know is refused on both sides
    MC_SCHEM_CHECK(
        not schem->save_to_file("./exported_test_data.unknown").has_value());
    MC_SCHEM_CHECK(not schematic::load_from_file("./exported_test_data.unknown")
                           .has_value());

    // save_to_file picks the format from the extension, and the result parses
    // back
    const pos_t shape = schem->shape();
    const uint64_t volume = schem->volume();
    const uint64_t solid = schem->total_blocks(false);

    MC_SCHEM_CHECK(
        schem->save_to_file("./exported_test_data.litematic").has_value());
    {
      auto reloaded = load_or_abort("./exported_test_data.litematic");
      MC_SCHEM_CHECK((reloaded->shape() == shape));
      MC_SCHEM_CHECK(reloaded->volume() == volume);
      MC_SCHEM_CHECK(reloaded->total_blocks(false) == solid);
      check_aggregates(*reloaded);
      check_first_hit(*reloaded);
    }

    MC_SCHEM_CHECK(
        schem->save_to_file("./exported_test_data.schem").has_value());
    {
      auto reloaded = load_or_abort("./exported_test_data.schem");
      MC_SCHEM_CHECK(reloaded->regions_count() >= 1);
      MC_SCHEM_CHECK(reloaded->volume() >= 1);
      check_aggregates(*reloaded);
    }

    MC_SCHEM_CHECK(schem->save_to_file("./exported_test_data.nbt").has_value());
    {
      auto reloaded = load_or_abort("./exported_test_data.nbt");
      MC_SCHEM_CHECK(reloaded->regions_count() >= 1);
      MC_SCHEM_CHECK(reloaded->volume() >= 1);
      check_aggregates(*reloaded);
    }
  }

  return 0;
}
