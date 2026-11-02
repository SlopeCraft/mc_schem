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

#include <unordered_set>
#include <random>
#include <vector>
#include <array>
#include <chrono>

#include <mc_schem.hpp>
#include "utils.hpp"

using namespace mc_schem;

struct hasher {
  static auto operator()(const std::array<int32_t, 3>& arr) noexcept {
    return std::hash<int32_t>()(arr[0]) xor std::hash<int32_t>()(arr[1]) xor
           std::hash<int32_t>()(arr[2]);
  }
};

int main(int argc, char** argv) {
  const size_t N_items_first = 10000;
  const size_t N_times_second = 10000000;
  const std::array shape{1000, 128, 1000};

  std::mt19937 mt{std::random_device{}()};

  auto reg = [shape] -> unique_region {
    auto reg = mc_schem::region::create(shape[0], shape[1], shape[2]);

    for (const auto id :
         {"minecraft:air", "minecraft:stone", "minecraft:glass"}) {
      auto blk = unwrap(block::create(id));
      auto _ = reg->find_or_append_to_palette(*blk);
    }
    reg->convert_to_sparse();
    return std::move(reg);
  }();
  assert(reg->palette_size() == 3);

  // prepare unduplicated insertions
  auto make_nodup_coordinates = [shape](const size_t count,
                                        std::mt19937& rand) {
    if (static_cast<double>(count) >= static_cast<double>(shape[0]) *
                                          static_cast<double>(shape[1]) *
                                          static_cast<double>(shape[2]) * 0.9) {
      throw std::runtime_error{"Too dense"};
    }

    std::unordered_set<std::array<int32_t, 3>, hasher> temp;
    temp.reserve(count);

    std::uniform_int_distribution<int32_t> rand_x{0, shape[0] - 1};
    std::uniform_int_distribution<int32_t> rand_y{0, shape[1] - 1};
    std::uniform_int_distribution<int32_t> rand_z{0, shape[2] - 1};

    while (true) {
      if (temp.size() >= count) break;

      const std::array<int32_t, 3> pos = {rand_x(rand), rand_y(rand),
                                          rand_z(rand)};
      temp.emplace(pos);
    }
    assert(temp.size() == count);

    std::vector<std::array<int32_t, 3>> result;
    result.reserve(count);
    for (const auto& pos : temp) {
      result.emplace_back(pos);
    }
    return result;
  };
  // first fill
  {
    auto poses = make_nodup_coordinates(N_items_first, mt);

    for (const auto& pos : poses) {
      reg->set_block(pos, 1);
    }
  }

  // second fill
  {
    const auto poses = make_nodup_coordinates(N_times_second, mt);
    auto reg_batch_insert = reg->clone();
    auto reg_sequential_insert = std::move(reg);

    {
      const auto begin = std::chrono::high_resolution_clock::now();
      for (const auto& pos : poses) {
        reg_sequential_insert->set_block(pos, 2);
      }
      const auto end = std::chrono::high_resolution_clock::now();
      const auto us =
          std::chrono::duration_cast<std::chrono::microseconds>(end - begin)
              .count();
      std::println("Insert {} elements by set_block takes {} ms", poses.size(),
                   static_cast<double>(us) / 1000);
    }
    {
      std::vector<uint16_t> indices;
      indices.resize(N_times_second, 2);
      const auto begin = std::chrono::high_resolution_clock::now();
      reg_batch_insert->set_block_batch(poses, indices);
      const auto end = std::chrono::high_resolution_clock::now();
      const auto us =
          std::chrono::duration_cast<std::chrono::microseconds>(end - begin)
              .count();
      std::println("Insert {} elements by set_block takes {} ms", poses.size(),
                   static_cast<double>(us) / 1000);
    }
  }

  return 0;
}