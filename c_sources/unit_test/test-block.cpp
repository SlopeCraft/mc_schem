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

#include <mc_schem.hpp>
#include <string>

#include "utils.hpp"

/// Coverage of the block wrapper. C functions behind every section:
///   mc_schem_create_block, mc_schem_create_block_from_common,
///   mc_schem_destroy_block (deleter of unique_block),
///   mc_schem_clone_block,
///   mc_schem_block_get_id, mc_schem_block_get_namespace,
///   mc_schem_block_set_id, mc_schem_block_set_namespace,
///   mc_schem_block_get_full_id, mc_schem_block_reset,
///   mc_schem_block_visit_attributes, mc_schem_block_erase_attribute,
///   mc_schem_block_set_attribute,
///   mc_schem_block_is_air, mc_schem_block_is_structure_void
int main(int, char**) {
  using namespace mc_schem;

  // mc_schem_create_block / mc_schem_destroy_block
  {
    auto blk = block::create();
    MC_SCHEM_CHECK(blk);

    // A freshly created block is `minecraft:air`
    MC_SCHEM_CHECK(blk->namespace_() == "minecraft");
    MC_SCHEM_CHECK(blk->id() == "air");
    MC_SCHEM_CHECK(blk->full_id() == "minecraft:air");
    MC_SCHEM_CHECK(blk->is_air());
    MC_SCHEM_CHECK(not blk->is_structure_void());
  }

  // mc_schem_create_block_from_common
  {
    auto air = block::from_common(common_block::air);
    MC_SCHEM_CHECK(air);
    MC_SCHEM_CHECK(air->is_air());
    MC_SCHEM_CHECK(not air->is_structure_void());
    MC_SCHEM_CHECK(air->full_id() == "minecraft:air");

    auto void_blk = block::from_common(common_block::structure_void);
    MC_SCHEM_CHECK(void_blk);
    MC_SCHEM_CHECK(void_blk->is_structure_void());
    MC_SCHEM_CHECK(not void_blk->is_air());
    MC_SCHEM_CHECK(void_blk->full_id() == "minecraft:structure_void");
  }

  // mc_schem_clone_block, including deep copy of the attribute map
  {
    auto src = block::create();
    src->set_id("oak_stairs");
    src->set_attribute("facing", "north");

    auto copy = src->clone();
    MC_SCHEM_CHECK(copy);
    MC_SCHEM_CHECK(copy->full_id() == "minecraft:oak_stairs[facing=north]");

    copy->set_id("spruce_stairs");
    copy->set_attribute("facing", "south");

    // src must be untouched by the modification of copy
    MC_SCHEM_CHECK(src->id() == "oak_stairs");
    MC_SCHEM_CHECK(src->full_id() == "minecraft:oak_stairs[facing=north]");
    MC_SCHEM_CHECK(copy->full_id() == "minecraft:spruce_stairs[facing=south]");
  }

  // mc_schem_block_get_id / mc_schem_block_set_id /
  // mc_schem_block_get_namespace / mc_schem_block_set_namespace
  {
    auto blk = block::create();

    blk->set_id("oak_log");
    blk->set_namespace("minecraft");
    MC_SCHEM_CHECK(blk->id() == "oak_log");
    MC_SCHEM_CHECK(blk->namespace_() == "minecraft");
    MC_SCHEM_CHECK(blk->full_id() == "minecraft:oak_log");
    MC_SCHEM_CHECK(not blk->is_air());
    MC_SCHEM_CHECK(not blk->is_structure_void());

    blk->set_namespace("my_pack");
    MC_SCHEM_CHECK(blk->namespace_() == "my_pack");
    MC_SCHEM_CHECK(blk->id() == "oak_log");
    MC_SCHEM_CHECK(blk->full_id() == "my_pack:oak_log");

    // An empty namespace is allowed and omitted from the full id
    blk->set_namespace("");
    MC_SCHEM_CHECK(blk->namespace_().empty());
    MC_SCHEM_CHECK(blk->full_id() == "oak_log");
  }

  // mc_schem_block_set_attribute / mc_schem_block_erase_attribute
  {
    auto blk = block::create();
    blk->set_id("oak_stairs");

    blk->set_attribute("facing", "north");
    blk->set_attribute("half", "bottom");
    // attributes are stored in a BTreeMap, so the full id is key-sorted
    MC_SCHEM_CHECK(blk->full_id() == "minecraft:oak_stairs[facing=north,half=bottom]");

    // Setting an existing key overwrites its value
    blk->set_attribute("half", "top");
    MC_SCHEM_CHECK(blk->full_id() == "minecraft:oak_stairs[facing=north,half=top]");

    blk->erase_attribute("half");
    MC_SCHEM_CHECK(blk->full_id() == "minecraft:oak_stairs[facing=north]");

    // Erasing a missing key is a no-op
    blk->erase_attribute("not_exist");
    MC_SCHEM_CHECK(blk->full_id() == "minecraft:oak_stairs[facing=north]");

    blk->erase_attribute("facing");
    MC_SCHEM_CHECK(blk->full_id() == "minecraft:oak_stairs");

    // mc_schem_block_is_air / mc_schem_block_is_structure_void:
    // a block with attributes is neither air nor structure void
    auto air_like = block::create();
    air_like->set_id("air");
    air_like->set_namespace("minecraft");
    air_like->set_attribute("foo", "bar");
    MC_SCHEM_CHECK(not air_like->is_air());

    auto void_like = block::from_common(common_block::structure_void);
    void_like->set_attribute("foo", "bar");
    MC_SCHEM_CHECK(not void_like->is_structure_void());
  }

  // mc_schem_block_visit_attributes
  {
    auto blk = block::create();
    blk->set_id("oak_stairs");
    blk->set_attribute("facing", "north");
    blk->set_attribute("half", "bottom");
    blk->set_attribute("waterlogged", "false");

    std::string visited;
    blk->visit_attributes([&visited](std::string&& key, std::string&& value) {
      if (not visited.empty()) {
        visited.push_back(',');
      }
      visited.append(key);
      visited.push_back('=');
      visited.append(value);
    });
    MC_SCHEM_CHECK(visited == "facing=north,half=bottom,waterlogged=false");

    // Visiting a block without attributes calls the callback zero times
    bool called = false;
    auto plain = block::create();
    plain->visit_attributes(
        [&called](std::string&&, std::string&&) { called = true; });
    MC_SCHEM_CHECK(not called);
  }

  // mc_schem_block_reset: success path
  {
    auto blk = block::create();

    const auto with_attrs =
        blk->reset("minecraft:oak_stairs[facing=north,half=top]");
    MC_SCHEM_CHECK(with_attrs.has_value());
    MC_SCHEM_CHECK(blk->namespace_() == "minecraft");
    MC_SCHEM_CHECK(blk->id() == "oak_stairs");
    MC_SCHEM_CHECK(blk->full_id() == "minecraft:oak_stairs[facing=north,half=top]");

    // The namespace may be omitted
    const auto no_namespace = blk->reset("stone");
    MC_SCHEM_CHECK(no_namespace.has_value());
    MC_SCHEM_CHECK(blk->namespace_().empty());
    MC_SCHEM_CHECK(blk->id() == "stone");
    MC_SCHEM_CHECK(blk->full_id() == "stone");

    // Reset drops the attributes of the previous value
    const auto dropped = blk->reset("minecraft:stone");
    MC_SCHEM_CHECK(dropped.has_value());
    MC_SCHEM_CHECK(blk->full_id() == "minecraft:stone");
  }

  // mc_schem_block_reset: failure path reports the parse error and keeps the
  // old value untouched
  {
    auto blk = block::create();
    const auto prepared = blk->reset("minecraft:stone");
    MC_SCHEM_CHECK(prepared.has_value());

    const auto too_many_colons = blk->reset("minecraft:stone:extra");
    MC_SCHEM_CHECK(not too_many_colons.has_value());
    MC_SCHEM_CHECK(too_many_colons.error() == block_id_parse_error::TooManyColons);

    const auto missing_equal = blk->reset("minecraft:stone[facing]");
    MC_SCHEM_CHECK(not missing_equal.has_value());
    MC_SCHEM_CHECK(missing_equal.error() ==
          block_id_parse_error::MissingEqualInAttributes);

    const auto invalid_char = blk->reset("Minecraft:stone");
    MC_SCHEM_CHECK(not invalid_char.has_value());
    MC_SCHEM_CHECK(invalid_char.error() == block_id_parse_error::InvalidCharacter);

    // None of the failed resets may change the block
    MC_SCHEM_CHECK(blk->namespace_() == "minecraft");
    MC_SCHEM_CHECK(blk->id() == "stone");
    MC_SCHEM_CHECK(blk->full_id() == "minecraft:stone");
  }

  return 0;
}
