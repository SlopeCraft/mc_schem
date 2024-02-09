#include <mc_schem.hpp>
#include <cassert>

int main(int argc, char **argv) {

  {
    const char id[] = "red_mushroom_block[east=true,west=true,north=true,south=true,up=true,down=true]";
    auto block = mc_schem::block::parse_block(id);
    assert(block);
    auto full_id = block.value()->full_id();
    assert(full_id == id);

    auto attributes = block.value()->attributes();
  }

  {

  }

  return 0;
}