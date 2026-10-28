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

#include <iostream>
#include <fstream>
#include <print>

#include <mc_schem.hpp>

int main(int argc, char** argv) {
  using namespace mc_schem;

  try {
    // Create a region from palette
    auto region = [] {
      std::vector<unique_block> palette;
      // Its strongly suggested to put air at index 0
      palette.emplace_back(block::create(common_block::air));
      palette.emplace_back(unwrap(block::create("minecraft:cauldron")));
      palette.emplace_back(unwrap(block::create("minecraft:glass")));

      std::vector<const block*> palette_view;
      palette_view.reserve(palette.size());
      for (auto& blk : palette) {
        palette_view.emplace_back(blk.get());
      }
      // Region will deep copy the palette
      auto region_exp =
          unwrap(region::create_with_palette({1, 3, 1}, palette_view));
      return std::move(region_exp);
    }();
    // Suggested usage. Setup an palette and simply fill with index
    region->set_block({0, 0, 0}, 1);  // cauldron
    region->set_block({0, 1, 0}, 2);  // glass
    // This also works, but slow. Don't use in loop
    {
      auto carpet_blk = unwrap(block::create("minecraft:white_carpet"));
      region->set_block({0, 2, 0}, *carpet_blk);  // carpet
    }

    // Create a surprising entity
    auto entity = [] {
      // Up to 2026.10, fastSNBT doesn't support true and spaces. sNBT have
      // to be tight
      // Rotation is also compulsory. Otherwise litematica crashes when pasting.
      auto tags = unwrap(nbt_hashmap::create_from_snbt(
          R"({id:"minecraft:creeper",Pos:[0.5d,0.25d,0.5d],Rotation:[0.0f,0.0f],ignited:1b,powered:1b})"));

      auto entity_exp = entity::create(std::move(tags));
      return std::move(*entity_exp);
    }();
    // Add our little cute
    region->add_entity(std::move(entity));
    // Set region name
    region->set_name(R"(我要把虫宝撕！成！两！半！)");

    // Create schematic
    auto schem = schematic::create();
    {
      // Java 26.3. Set up metadata
      auto md = unwrap(metadata_ir::create(5023));
      md->set_name("虫宝撕两半儿机");
      md->set_author("TokiNoBug");
      md->set_description("全物品必备");
      schem->set_metadata(std::move(md));
    }
    // Move our region into schematic
    schem->append_region(std::move(region));

    {
      std::ofstream lite{"creeper-cpp.litematic", std::ios::binary};
      litematica_save_option option{};
      unwrap(schem->save_litematica(lite, option));
    }
    {
      std::ofstream vs{"creeper-cpp.nbt", std::ios::binary};
      vanilla_structure_save_option option{};
      unwrap(schem->save_vanilla_structure(vs, option));
    }
    {
      std::ofstream we{"creeper-cpp.schem", std::ios::binary};
      world_edit13_save_option option{};
      unwrap(schem->save_world_edit13(we, option));
    }

  } catch (const std::exception& e) {
    std::println(std::cerr, "Exception: {}", e.what());
    return 1;
  }

  return 0;
}