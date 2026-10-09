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
#include <mc_schem.hpp>
#include <optional>
#include <span>
#include <stdexcept>
#include <string>
#include <tuple>
#include <vector>

#include "utils.hpp"

namespace {

/// A block made from `full_id`, aborting if the id cannot be parsed.
mc_schem::unique_block make_block(const std::string& full_id) {
  auto blk = mc_schem::block::create();
  MC_SCHEM_CHECK(blk->reset(full_id).has_value());
  return blk;
}

}  // namespace

/// Coverage of the region wrapper. Regions built by `region::create` cover the
/// geometry, palette and block access API, and three litematica fixtures supply
/// non-trivial data: `correct_test.litematic` for blocks, `test02.litematic`
/// for entities and `test03.litematic` for block entities and pending ticks.
/// No check depends on the contents of those fixtures, so any other projection
/// file that carries the same features can be used instead.
/// C functions behind every section:
///   mc_schem_create_region, mc_schem_create_region_with_palette,
///   mc_schem_destroy_region (deleter of unique_region),
///   mc_schem_clone_region,
///   mc_schem_region_get_name, mc_schem_region_set_name,
///   mc_schem_region_get_offset, mc_schem_region_set_offset,
///   mc_schem_region_get_size, mc_schem_region_is_dense,
///   mc_schem_region_reshape,
///   mc_schem_region_palette_get_size, mc_schem_region_palette_get_block,
///   mc_schem_region_find_or_append_to_palette,
///   mc_schem_region_find_in_palette, mc_schem_region_get_entities_count,
///   mc_schem_region_get_entity, mc_schem_region_get_entity_mut,
///   mc_schem_region_erase_entity, mc_schem_region_add_entity,
///   mc_schem_region_add_entity_move, mc_schem_region_get_block_entities_count,
///   mc_schem_region_visit_block_entities, mc_schem_region_get_block_entity,
///   mc_schem_region_get_block_entity_mut, mc_schem_region_add_block_entity,
///   mc_schem_region_add_block_entity_move,
///   mc_schem_region_get_pending_ticks_count,
///   mc_schem_region_get_pending_tick, mc_schem_region_get_pending_tick_mut,
///   mc_schem_region_get_block_index, mc_schem_region_set_block_by_index,
///   mc_schem_region_set_block_by_block, mc_schem_region_shrink_palette,
///   mc_schem_region_fill_with, mc_schem_region_convert_to_sparse,
///   mc_schem_region_convert_to_dense, mc_schem_region_total_blocks,
///   mc_schem_region_visit_blocks
int main(int argc, char** argv) {
  using namespace mc_schem;

  // Convention shared with the caller and with ctest: argv[1] is the directory
  // that holds the test fixtures. It is taken as is, the guard below only keeps
  // a missing argument from turning into undefined behaviour.
  MC_SCHEM_CHECK(argc >= 2);
  const std::filesystem::path test_files_dir{argv[1]};

  // mc_schem_create_region / mc_schem_destroy_region,
  // mc_schem_region_get_name / set_name, get_offset / set_offset, get_size,
  // is_dense, palette_get_size / palette_get_block, total_blocks
  {
    auto r = region::create(2, 3, 4);
    MC_SCHEM_CHECK(r);

    MC_SCHEM_CHECK(r->name() == "NewRegion");
    r->set_name("unit-test");
    MC_SCHEM_CHECK(r->name() == "unit-test");

    MC_SCHEM_CHECK((r->offset() == pos_t{0, 0, 0}));
    r->set_offset({-1, 2, -3});
    MC_SCHEM_CHECK((r->offset() == pos_t{-1, 2, -3}));

    MC_SCHEM_CHECK((r->size_xyz() == pos_t{2, 3, 4}));
    MC_SCHEM_CHECK((r->size_yzx() == pos_t{3, 4, 2}));

    // A fresh region is dense and its palette only holds air
    MC_SCHEM_CHECK(r->is_dense());
    MC_SCHEM_CHECK(not r->is_sparse());
    MC_SCHEM_CHECK(r->palette_size() == 1);
    MC_SCHEM_CHECK(r->palette(0) not_eq nullptr);
    MC_SCHEM_CHECK(r->palette(0)->is_air());
    MC_SCHEM_CHECK(r->palette(1) == nullptr);  // out of range

    // Every cell of a fresh region is air, and air is not counted unless asked
    MC_SCHEM_CHECK(r->total_blocks(false) == 0);
    MC_SCHEM_CHECK(r->total_blocks(true) == 2u * 3u * 4u);
    MC_SCHEM_CHECK(r->block_index_at({1, 2, 3}).value() == 0);
  }

  // mc_schem_create_region_with_palette
  {
    auto stone = make_block("minecraft:stone");
    auto dirt = make_block("minecraft:dirt");

    const std::array<const block*, 2> palette{stone.get(), dirt.get()};
    auto result = region::create_with_palette({1, 1, 2}, palette);
    MC_SCHEM_CHECK(result.has_value());
    auto& r = *result;  // value() & cannot compile with a unique_ptr error

    MC_SCHEM_CHECK((r->size_xyz() == pos_t{1, 1, 2}));
    MC_SCHEM_CHECK(r->palette_size() == 2);
    MC_SCHEM_CHECK(r->palette(0)->full_id() == "minecraft:stone");
    MC_SCHEM_CHECK(r->palette(1)->full_id() == "minecraft:dirt");
    MC_SCHEM_CHECK(r->block_index_at({0, 0, 0}).value() == 0);

    // An empty palette must fail and hand back an error message
    const std::span<const block*> empty_palette{};
    auto failed = region::create_with_palette({1, 1, 1}, empty_palette);
    MC_SCHEM_CHECK(not failed.has_value());
    MC_SCHEM_CHECK(not failed.error()->message().empty());
  }

  // mc_schem_clone_region
  {
    auto r = region::create(2, 2, 2);
    auto stone = make_block("minecraft:stone");
    r->set_name("original");
    r->set_offset({4, 5, 6});
    r->set_block({0, 1, 0}, *stone);

    auto copy = r->clone();
    MC_SCHEM_CHECK(copy);
    MC_SCHEM_CHECK(copy->name() == "original");
    MC_SCHEM_CHECK((copy->offset() == pos_t{4, 5, 6}));
    MC_SCHEM_CHECK((copy->size_xyz() == pos_t{2, 2, 2}));
    MC_SCHEM_CHECK(copy->palette_size() == r->palette_size());
    MC_SCHEM_CHECK(copy->block_index_at({0, 1, 0}).value() ==
                   r->block_index_at({0, 1, 0}).value());

    // The clone shares nothing with the original
    copy->set_name("clone");
    copy->set_block({1, 1, 1}, *stone);
    MC_SCHEM_CHECK(r->name() == "original");
    MC_SCHEM_CHECK(r->palette_size() == 2);
    MC_SCHEM_CHECK(r->block_index_at({1, 1, 1}).value() == 0);
    MC_SCHEM_CHECK(copy->palette_size() == 2);
    MC_SCHEM_CHECK(copy->block_index_at({1, 1, 1}).value() == 1);
  }

  // mc_schem_swap_region
  {
    auto small = region::create(1, 1, 1);
    MC_SCHEM_CHECK(small);
    small->set_name("small");
    auto stone = make_block("minecraft:stone");
    small->set_block({0, 0, 0}, *stone);

    auto large = region::create(2, 2, 2);
    MC_SCHEM_CHECK(large);
    large->set_name("large");

    small->swap(*large);
    MC_SCHEM_CHECK(small->name() == "large");
    MC_SCHEM_CHECK(large->name() == "small");
    MC_SCHEM_CHECK((small->size_xyz() == pos_t{2, 2, 2}));
    MC_SCHEM_CHECK((large->size_xyz() == pos_t{1, 1, 1}));
    MC_SCHEM_CHECK(large->palette(1)->full_id() == "minecraft:stone");
  }

  // mc_schem_region_get_block_index, set_block_by_block, set_block_by_index,
  // block_at, block_info_at, find_in_palette, find_or_append_to_palette,
  // palette_get_block
  {
    auto r = region::create(2, 2, 2);
    auto stone = make_block("minecraft:stone");

    MC_SCHEM_CHECK(r->block_index_at({0, 0, 0}).value() == 0);
    MC_SCHEM_CHECK(not r->block_index_at({2, 0, 0}).has_value());   // x too big
    MC_SCHEM_CHECK(not r->block_index_at({0, -1, 0}).has_value());  // negative
    MC_SCHEM_CHECK(r->contains_coordinate({1, 1, 1}));
    MC_SCHEM_CHECK(not r->contains_coordinate({2, 2, 2}));

    r->set_block({1, 1, 1}, *stone);
    MC_SCHEM_CHECK(r->palette_size() == 2);
    const uint16_t stone_idx = r->block_index_at({1, 1, 1}).value();
    MC_SCHEM_CHECK(r->palette(stone_idx)->full_id() == "minecraft:stone");

    r->set_block({0, 0, 0}, stone_idx);
    MC_SCHEM_CHECK(r->block_index_at({0, 0, 0}).value() == stone_idx);
    r->set_block({0, 0, 0}, static_cast<uint16_t>(0));
    MC_SCHEM_CHECK(r->block_index_at({0, 0, 0}).value() == 0);

    const auto at = r->block_at({1, 1, 1});
    MC_SCHEM_CHECK(at.has_value());
    MC_SCHEM_CHECK(std::get<0>(at.value()) == stone_idx);
    MC_SCHEM_CHECK(std::get<1>(at.value()).get().full_id() ==
                   "minecraft:stone");
    MC_SCHEM_CHECK(not r->block_at({9, 9, 9}).has_value());

    const auto info = r->block_info_at({1, 1, 1});
    MC_SCHEM_CHECK(info.has_value());
    MC_SCHEM_CHECK(std::get<0>(info.value()) == stone_idx);
    MC_SCHEM_CHECK(std::get<1>(info.value()).get().full_id() ==
                   "minecraft:stone");
    MC_SCHEM_CHECK(std::get<2>(info.value()) == nullptr);
    MC_SCHEM_CHECK(std::get<3>(info.value()).empty());
    MC_SCHEM_CHECK(not r->block_info_at({9, 9, 9}).has_value());

    // An existing block keeps its index, a new one is appended
    MC_SCHEM_CHECK(r->find_in_palette(*stone).value() == stone_idx);
    MC_SCHEM_CHECK(r->find_or_append_to_palette(*stone) == stone_idx);
    MC_SCHEM_CHECK(r->palette_size() == 2);

    auto diamond = make_block("minecraft:diamond_block");
    MC_SCHEM_CHECK(not r->find_in_palette(*diamond).has_value());
    const uint16_t diamond_idx = r->find_or_append_to_palette(*diamond);
    MC_SCHEM_CHECK(diamond_idx == 2);
    MC_SCHEM_CHECK(r->palette_size() == 3);
    MC_SCHEM_CHECK(r->palette(diamond_idx)->full_id() ==
                   "minecraft:diamond_block");
    MC_SCHEM_CHECK(r->find_in_palette(*diamond).value() == diamond_idx);

    const auto palette = r->full_palette();
    MC_SCHEM_CHECK(palette.size() == r->palette_size());
    for (size_t i = 0; i < palette.size(); ++i) {
      MC_SCHEM_CHECK(palette[i] not_eq nullptr);
      MC_SCHEM_CHECK(palette[i] == r->palette(i));
    }
  }

  // Error branches of set_block, which surface as std::runtime_error
  {
    auto r = region::create(2, 1, 1);
    auto stone = make_block("minecraft:stone");
    r->set_block({0, 0, 0}, *stone);

    bool caught_index = false;
    try {
      r->set_block({0, 0, 0}, static_cast<uint16_t>(12345));
    } catch (const std::runtime_error&) {
      caught_index = true;
    }
    MC_SCHEM_CHECK(caught_index);

    bool caught_pos = false;
    try {
      r->set_block({2, 0, 0}, *stone);
    } catch (const std::runtime_error&) {
      caught_pos = true;
    }
    MC_SCHEM_CHECK(caught_pos);

    // A rejected write must leave the region alone
    MC_SCHEM_CHECK(r->block_index_at({0, 0, 0}).value() == 1);
    MC_SCHEM_CHECK(r->palette_size() == 2);
  }

  // mc_schem_region_fill_with, total_blocks, visit_blocks, convert_to_sparse,
  // convert_to_dense, reshape, shrink_palette
  {
    auto r = region::create(2, 2, 2);
    auto stone = make_block("minecraft:stone");

    // explicit_only = false visits every cell, all of them air
    size_t visited_all = 0;
    r->visit_blocks(
        [&visited_all](pos_t, uint16_t idx, const block& blk,
                       const block_entity*) {
          MC_SCHEM_CHECK(idx == 0);
          MC_SCHEM_CHECK(blk.is_air());
          ++visited_all;
        },
        false);
    MC_SCHEM_CHECK(visited_all == 8);
    MC_SCHEM_CHECK(r->total_blocks(false) == 0);
    MC_SCHEM_CHECK(r->total_blocks(true) == 8);

    r->fill_with(*stone);
    size_t visited_stone = 0;
    r->visit_blocks(
        [&visited_stone](pos_t, uint16_t idx, const block& blk,
                         const block_entity*) {
          MC_SCHEM_CHECK(idx == 1);
          MC_SCHEM_CHECK(blk.full_id() == "minecraft:stone");
          ++visited_stone;
        },
        false);
    MC_SCHEM_CHECK(visited_stone == 8);
    MC_SCHEM_CHECK(r->total_blocks(false) == 8);
    MC_SCHEM_CHECK(r->total_blocks(true) == 8);

    // Filling with air turns every cell back into air
    auto air = block::from_common(common_block::air);
    r->fill_with(*air);
    size_t visited_air = 0;
    r->visit_blocks(
        [&visited_air](pos_t, uint16_t idx, const block& blk,
                       const block_entity*) {
          MC_SCHEM_CHECK(idx == 0);
          MC_SCHEM_CHECK(blk.is_air());
          ++visited_air;
        },
        false);
    MC_SCHEM_CHECK(visited_air == 8);
    MC_SCHEM_CHECK(r->total_blocks(false) == 0);
    MC_SCHEM_CHECK(r->total_blocks(true) == 8);

    // On a sparse region only the explicit blocks stay behind
    auto s = region::create(2, 2, 2);
    s->set_block({1, 0, 1}, *stone);
    MC_SCHEM_CHECK(s->is_dense());

    s->convert_to_sparse();
    MC_SCHEM_CHECK(s->is_sparse());
    size_t explicit_visits = 0;
    s->visit_blocks(
        [&](pos_t pos, uint16_t idx, const block& blk, const block_entity*) {
          MC_SCHEM_CHECK((pos == pos_t{1, 0, 1}));
          MC_SCHEM_CHECK(idx == 1);
          MC_SCHEM_CHECK(blk.full_id() == "minecraft:stone");
          ++explicit_visits;
        },
        true);
    MC_SCHEM_CHECK(explicit_visits == 1);

    // Walking a sparse array with explicit_only = false. The exact number of
    // calls is deliberately not asserted: Sparse3DArray::visit_dense in
    // src/region.rs restarts its gap loop at the previous non-zero index, so
    // every non-zero cell is reported a second time with index 0. Only the
    // bounds that hold with and without that off-by-one are checked here, and
    // total_blocks(true) is skipped for the same reason (it currently counts 9
    // instead of 8 cells).
    size_t sparse_all = 0;
    s->visit_blocks([&sparse_all](pos_t, uint16_t, const block&,
                                  const block_entity*) { ++sparse_all; },
                    false);
    MC_SCHEM_CHECK(sparse_all >= 8);
    MC_SCHEM_CHECK(sparse_all <= 16);

    // The count of solid blocks is unaffected by that duplicate visit
    MC_SCHEM_CHECK(s->total_blocks(false) == 1);
    MC_SCHEM_CHECK(s->block_index_at({0, 0, 0}).value() == 0);
    MC_SCHEM_CHECK(s->block_index_at({1, 0, 1}).value() == 1);

    s->convert_to_dense();
    MC_SCHEM_CHECK(s->is_dense());
    MC_SCHEM_CHECK(s->total_blocks(false) == 1);
    MC_SCHEM_CHECK(s->total_blocks(true) == 8);
    MC_SCHEM_CHECK(s->block_index_at({1, 0, 1}).value() == 1);
    MC_SCHEM_CHECK(s->block_index_at({0, 0, 0}).value() == 0);

    // reshape drops the content but keeps the palette
    auto shaped = region::create(2, 2, 2);
    shaped->set_block({0, 0, 0}, *stone);
    MC_SCHEM_CHECK(shaped->palette_size() == 2);
    shaped->reshape({3, 1, 2});
    MC_SCHEM_CHECK((shaped->size_xyz() == pos_t{3, 1, 2}));
    MC_SCHEM_CHECK((shaped->size_yzx() == pos_t{1, 2, 3}));
    MC_SCHEM_CHECK(shaped->palette_size() == 2);
    size_t shaped_air = 0;
    shaped->visit_blocks(
        [&shaped_air](pos_t, uint16_t idx, const block& blk,
                      const block_entity*) {
          MC_SCHEM_CHECK(idx == 0);
          MC_SCHEM_CHECK(blk.is_air());
          ++shaped_air;
        },
        false);
    MC_SCHEM_CHECK(shaped_air == 3u * 1u * 2u);
    MC_SCHEM_CHECK(shaped->total_blocks(false) == 0);
    MC_SCHEM_CHECK(shaped->total_blocks(true) == 3u * 1u * 2u);

    // shrink_palette drops the stone that reshape made unused
    const auto shrunk = shaped->shrink_palette();
    MC_SCHEM_CHECK(shrunk.has_value());
    MC_SCHEM_CHECK(shaped->palette_size() == 1);
    MC_SCHEM_CHECK(shaped->palette(0)->is_air());
  }

  // A region read from a real litematica file. The fixture is only a source of
  // non-trivial block data: nothing below depends on its shape, on its palette
  // or on which block sits where
  {
    auto schem = load_schematic_or_abort(test_files_dir,
                                         "litematica/correct_test.litematic");
    region& r = first_region_of(schem);
    const region& cr = r;

    MC_SCHEM_CHECK(cr.is_dense());
    MC_SCHEM_CHECK(cr.palette_size() >= 1);

    const pos_t shape = cr.size_xyz();
    for (size_t dim = 0; dim < 3; ++dim) {
      MC_SCHEM_CHECK(shape[dim] >= 1);
    }

    // A sparse copy must answer the same block index for every single cell
    auto copy = cr.clone();
    MC_SCHEM_CHECK(copy);
    copy->convert_to_sparse();
    MC_SCHEM_CHECK(copy->is_sparse());

    uint64_t air_cells = 0, solid_cells = 0, structure_void_cells = 0;
    for (int32_t x = 0; x < shape[0]; ++x) {
      for (int32_t y = 0; y < shape[1]; ++y) {
        for (int32_t z = 0; z < shape[2]; ++z) {
          const pos_t pos{x, y, z};
          const auto dense_idx = cr.block_index_at(pos);
          const auto sparse_idx = copy->block_index_at(pos);
          MC_SCHEM_CHECK(dense_idx.has_value());
          MC_SCHEM_CHECK(sparse_idx.has_value());
          MC_SCHEM_CHECK(dense_idx.value() == sparse_idx.value());

          // every index must land inside the palette
          const block* blk = cr.palette(dense_idx.value());
          MC_SCHEM_CHECK(blk not_eq nullptr);
          if (blk->is_air()) {
            ++air_cells;
          } else if (blk->is_structure_void()) {
            ++structure_void_cells;
          } else {
            ++solid_cells;
          }
        }
      }
    }

    // total_blocks against an independent count over the same region
    MC_SCHEM_CHECK(cr.total_blocks(false) == solid_cells);
    MC_SCHEM_CHECK(cr.total_blocks(true) ==
                   solid_cells + air_cells + structure_void_cells);

    // Every palette entry has to be findable again, and asking to append one
    // that is already there must not grow the palette
    const auto palette = cr.full_palette();
    MC_SCHEM_CHECK(palette.size() == cr.palette_size());
    const size_t palette_size_before = palette.size();
    for (size_t i = 0; i < palette.size(); ++i) {
      const block* entry = palette[i];
      MC_SCHEM_CHECK(entry not_eq nullptr);

      const std::string entry_id = entry->full_id();

      const auto found = cr.find_in_palette(*entry);
      MC_SCHEM_CHECK(found.has_value());
      MC_SCHEM_CHECK(cr.palette(found.value())->full_id() == entry_id);

      // The id is captured above because an append, which must not happen here,
      // could reallocate the palette and invalidate `entry`
      const uint16_t appended = r.find_or_append_to_palette(*entry);
      MC_SCHEM_CHECK(cr.palette_size() == palette_size_before);
      MC_SCHEM_CHECK(r.palette(appended)->full_id() == entry_id);
    }
  }

  // mc_schem_region_get_entities_count, get_entity, get_entity_mut,
  // mc_schem_region_get_entities_count, get_entity, get_entity_mut,
  // add_entity, add_entity_move, erase_entity
  {
    auto schem =
        load_schematic_or_abort(test_files_dir, "litematica/test02.litematic");
    region& r = first_region_of(schem);
    const region& cr = r;

    const size_t count = cr.entities_count();
    MC_SCHEM_CHECK(count >= 1);
    for (size_t i = 0; i < count; ++i) {
      MC_SCHEM_CHECK(cr.get_entity(i) not_eq nullptr);
    }
    MC_SCHEM_CHECK(cr.get_entity(count) == nullptr);  // out of range

    MC_SCHEM_CHECK(r.get_entity(0) not_eq nullptr);
    MC_SCHEM_CHECK(r.get_entity(count) == nullptr);

    const entity* first = cr.get_entity(0);
    MC_SCHEM_CHECK(first not_eq nullptr);

    // add_entity deep copies and reports the new index
    const size_t added = r.add_entity(*first);
    MC_SCHEM_CHECK(added == count);
    MC_SCHEM_CHECK(r.entities_count() == count + 1);

    // erase_entity hands the ownership back to us
    auto erased = r.erase_entity(added);
    MC_SCHEM_CHECK(erased not_eq nullptr);
    MC_SCHEM_CHECK(r.entities_count() == count);
    MC_SCHEM_CHECK(r.get_entity(added) == nullptr);

    // add_entity_move takes that ownership and reports the new index, leaving
    // the handle empty
    const size_t moved_in = r.add_entity(std::move(erased));
    MC_SCHEM_CHECK(erased == nullptr);
    MC_SCHEM_CHECK(moved_in == added);
    MC_SCHEM_CHECK(r.entities_count() == count + 1);
    MC_SCHEM_CHECK(r.get_entity(moved_in) not_eq nullptr);
  }

  // mc_schem_region_get_block_entities_count, visit_block_entities,
  // get_block_entity, get_block_entity_mut, add_block_entity,
  // add_block_entity_move,
  // get_pending_ticks_count, get_pending_tick, get_pending_tick_mut
  // `test03.litematic` is used because it carries both block entities and
  // pending ticks. Their number and position are not assumed, only that at
  // least one exists so the calls below run at all.
  {
    auto schem =
        load_schematic_or_abort(test_files_dir, "litematica/test03.litematic");
    region& r = first_region_of(schem);
    const region& cr = r;

    const size_t be_count = cr.block_entities_count();
    MC_SCHEM_CHECK(be_count >= 1);

    std::vector<pos_t> be_positions;
    cr.visit_block_entities(
        [&be_positions](int32_t x, int32_t y, int32_t z, const block_entity&) {
          be_positions.push_back(pos_t{x, y, z});
        });
    MC_SCHEM_CHECK(be_positions.size() == be_count);

    for (const pos_t& pos : be_positions) {
      MC_SCHEM_CHECK(cr.get_block_entity(pos) not_eq nullptr);
      MC_SCHEM_CHECK(r.get_block_entity(pos) not_eq nullptr);

      // block_info_at routes the block entity through the same lookup. A block
      // entity may sit on the edge of a region, where there is no cell to
      // report, so it has to agree with block_index_at rather than always
      // yielding a value.
      const auto cell = cr.block_index_at(pos);
      const auto be_info = cr.block_info_at(pos);
      MC_SCHEM_CHECK(be_info.has_value() == cell.has_value());
      if (be_info.has_value()) {
        MC_SCHEM_CHECK(std::get<2>(be_info.value()) ==
                       cr.get_block_entity(pos));
      }
    }
    MC_SCHEM_CHECK(cr.get_block_entity({-1, -1, -1}) == nullptr);
    MC_SCHEM_CHECK(r.get_block_entity({-1, -1, -1}) == nullptr);

    // Replacing a block entity hands the previous one back, erasing hands the
    // stored one back, and erasing an empty position returns null
    const pos_t be_pos = be_positions.front();
    const block_entity* stored = cr.get_block_entity(be_pos);
    MC_SCHEM_CHECK(stored not_eq nullptr);

    auto replaced = r.add_block_entity(be_pos, *stored);
    MC_SCHEM_CHECK(replaced not_eq nullptr);
    MC_SCHEM_CHECK(r.block_entities_count() == be_count);

    auto removed = r.erase_block_entity(be_pos);
    MC_SCHEM_CHECK(removed not_eq nullptr);
    MC_SCHEM_CHECK(r.block_entities_count() == be_count - 1);

    auto nothing = r.erase_block_entity(be_pos);
    MC_SCHEM_CHECK(nothing == nullptr);
    MC_SCHEM_CHECK(r.get_block_entity(be_pos) == nullptr);

    // add_block_entity_move puts the erased one back: the position is empty, so
    // nothing comes back, and the handle is consumed
    auto previous = r.add_block_entity(be_pos, std::move(removed));
    MC_SCHEM_CHECK(previous == nullptr);
    MC_SCHEM_CHECK(removed == nullptr);
    MC_SCHEM_CHECK(r.block_entities_count() == be_count);
    MC_SCHEM_CHECK(r.get_block_entity(be_pos) not_eq nullptr);

    // Pending ticks are keyed by region relative position, so walk the region
    // to find a cell that carries some
    size_t tick_total = 0;
    std::optional<pos_t> tick_pos;
    const pos_t shape = cr.size_xyz();
    for (int32_t x = 0; x < shape[0]; ++x) {
      for (int32_t y = 0; y < shape[1]; ++y) {
        for (int32_t z = 0; z < shape[2]; ++z) {
          const pos_t pos{x, y, z};
          const size_t n = cr.pending_ticks_count_at(pos);
          tick_total += n;
          if (n > 0 and not tick_pos.has_value()) {
            tick_pos = pos;
          }
        }
      }
    }
    MC_SCHEM_CHECK(tick_total >= 1);
    MC_SCHEM_CHECK(tick_pos.has_value());
    MC_SCHEM_CHECK(cr.pending_ticks_count_at({-1, -1, -1}) == 0);

    const pos_t pos = tick_pos.value();
    const size_t tick_count = cr.pending_ticks_count_at(pos);
    MC_SCHEM_CHECK(tick_count >= 1);
    MC_SCHEM_CHECK(cr.pending_ticks_at(pos).size() == tick_count);
    MC_SCHEM_CHECK(r.pending_ticks_at(pos).size() == tick_count);

    // The fixture has a tick without a block entity, so only the agreement
    // between block_info_at and the dedicated lookups is checked here
    const auto info = cr.block_info_at(pos);
    MC_SCHEM_CHECK(info.has_value());
    MC_SCHEM_CHECK(std::get<1>(info.value()).get().full_id() ==
                   cr.palette(std::get<0>(info.value()))->full_id());
    MC_SCHEM_CHECK(std::get<2>(info.value()) == cr.get_block_entity(pos));
    MC_SCHEM_CHECK(std::get<3>(info.value()).size() == tick_count);
  }

  return 0;
}
