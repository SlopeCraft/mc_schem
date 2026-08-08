//
// Created by Joseph on 2026/8/6.
//

#include <mc_schem.hpp>
#include <print>

using namespace mc_schem;
void test_blocks() {
  auto block = block::create();

  std::println(R"(id of default block: "{}", namespace: "{}")", block->id(),
               block->namespace_());

  block->set_namespace("gia");
  block->set_id("lalala");

  std::println(R"(update block: "{}", namespace: "{}")", block->id(),
               block->namespace_());

  block->reset("minecraft:stone[gia=ho,nya=hia]").value();
  std::println(R"(update block: "{}")", block->full_id());

  std::print("Attributes: [");
  block->visit_attributes([](const std::string& k, const std::string& v) {
    std::print("{}={},", k, v);
  });
  std::println("]");

}

void test_region() {
  auto region = region::create(3, 4, 5);

  std::println("Palette: [");
  for (auto blk : region->full_palette()) {
    std::println("{}", blk->full_id());
  }
  std::println("]");
}

int main(int argc, char** argv) {
  test_blocks();
  test_region();
  return 0;
}