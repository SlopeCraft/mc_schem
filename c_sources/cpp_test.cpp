#include <mc_schem.hpp>
#include <cassert>
#include <format>
#include <print>

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
      std::print("{} = {}, ", it.key(), std::string_view{val});
    }
    assert(attributes.remove("no_such_key") == false);
    assert(attributes.remove("down") == true);
    assert(attributes.size() == 5);
  }

  {
    mc_schem::nbt *nbt;
  }

  return 0;
}