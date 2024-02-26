/*
mc_schem is a rust library to generate, load, manipulate and save minecraft
schematic files. Copyright (C) 2024  joseph

This program is free software: you can redistribute it and/or modify it under
the terms of the GNU General Public License as published by the Free Software
Foundation, either version 3 of the License, or (at your option) any later
version.

This program is distributed in the hope that
it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty
of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU General
Public License for more details.

You should have received a copy of the GNU General Public License
along with this program.  If not, see <https://www.gnu.org/licenses/>.
*/

#include <mc_schem.hpp>
#include <cassert>
#include <format>
#include <iterator>
#include <iostream>

int main(int argc, char **argv) {

  {
    std::string_view id = "red_mushroom_block[down=true,east=true,north=true,south=true,up=true,west=true]";
    auto block = mc_schem::block::parse_block(id);
    assert(block);
    auto full_id = block.value()->full_id();
    assert(full_id.size() == id.size());

    auto attributes = block.value()->attributes();
    for (auto it = attributes.begin(); it != attributes.end(); ++it) {
      auto val = it.value();
      std::format_to(std::ostream_iterator<char>{std::cout},
                     "{} = {}, ", it.key(), std::string_view{val});
    }
    assert(attributes.remove("no_such_key") == false);
    assert(attributes.remove("down") == true);
    assert(attributes.size() == 5);
  }


  return 0;
}