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

#ifndef MC_SCHEM_MC_SCHEM_HPP
#define MC_SCHEM_MC_SCHEM_HPP

#include <array>
#include <cassert>
#include <cstddef>
#include <cstdint>
#include <expected>
#include <memory>
#include <optional>
#include <span>
#include <string>
#include <string_view>
#include <type_traits>
#include <vector>

#include "mc_schem.hpp"

namespace mc_schem {

enum class block_id_parse_error : uint8_t {
  TooManyColons = 0,
  TooManyLeftBrackets = 1,
  TooManyRightBrackets = 2,
  MissingBlockId = 3,
  BracketsNotInPairs = 4,
  BracketInWrongPosition = 5,
  ColonsInWrongPosition = 6,
  MissingEqualInAttributes = 7,
  TooManyEqualsInAttributes = 8,
  MissingAttributeName = 9,
  MissingAttributeValue = 10,
  ExtraStringAfterRightBracket = 11,
  InvalidCharacter = 12,
};

// Forward declarations
class block;
class error;
class entity;
class block_entity;
class pending_tick;
class region;
class schematic;

/// Copy rust str to std::string
struct rust_string_receiver {
 private:
  static void callback_receive_string(const char8_t* str, size_t bytes,
                                      void* custom_data) {
    auto dest = reinterpret_cast<std::string*>(custom_data);
    std::u8string_view u8sv{str, bytes};
    dest->assign(u8sv.begin(), u8sv.end());
  }

 public:
  void (*func_receive_string)(const char8_t* str, size_t bytes,
                              void* custom_data){nullptr};
  void* custom_data{nullptr};

  explicit rust_string_receiver(std::string& dest)
      : func_receive_string{callback_receive_string}, custom_data{&dest} {}
};

extern "C" {
// create/destroy
[[nodiscard]] block* mc_schem_create_block();
void mc_schem_destroy_block(block* block);
void mc_schem_destroy_error(error*);
[[nodiscard]] region* mc_schem_create_region(int32_t size_x, int32_t size_y,
                                             int32_t size_z);
/// Create region with given palette. If error, error_dest is box of error and returns null;
/// Otherwise error_dest is null.
[[nodiscard]] region* mc_schem_create_region_with_palette(
    int32_t size_x, int32_t size_y, int32_t size_z,
    const block* const palette[], size_t palette_size, error** error_dest);
void mc_schem_destroy_region(region* region);

// Block
void mc_schem_block_get_id(const block*, const rust_string_receiver* receiver);
void mc_schem_block_get_namespace(const block*,
                                  const rust_string_receiver* receiver);
void mc_schem_block_set_id(block*, const char*);
void mc_schem_block_set_namespace(block*, const char*);
void mc_schem_block_get_full_id(const block* block,
                                const rust_string_receiver* receiver);
bool mc_schem_block_reset(block* block, const char* full_id,
                          block_id_parse_error* detail_nullable);
void mc_schem_block_visit_attributes(
    const block* block,
    void (*callback)(const char8_t* key, size_t key_bytes, const char8_t* value,
                     size_t value_bytes, void* custom_data),
    void* custom_data);
void mc_schem_block_erase_attribute(block* block, const char* key);
void mc_schem_block_set_attribute(block* block, const char* key,
                                  const char* value);
bool mc_schem_block_is_air(const block*);
bool mc_schem_block_is_structure_void(const block*);
void mc_schem_error_get_message(const error*,
                                const rust_string_receiver* receiver);

// Region
void mc_schem_region_get_name(const region*, const rust_string_receiver* dest);
void mc_schem_region_get_offset(const region*, int32_t* dest_x, int32_t* dest_y,
                                int32_t* dest_z);
void mc_schem_region_get_size(const region*, int32_t* size_x, int32_t* size_y,
                              int32_t* size_z);
size_t mc_schem_region_palette_get_size(const region*);
const block* mc_schem_region_palette_get_block(const region*, size_t index);
/// Add block into palette (deep copy). If identical block already exist in
/// palette, don't copy; otherwise append. Returns index of this block in palette
uint16_t mc_schem_region_find_or_append_to_palette(region* region,
                                                   const block* block);

// size_t mc_schem_region_get_entities_count(const region*);

// void mc_schem_region_visit_block_entities(const region*,
//                                           void (*callback)(int32_t x, int32_t
//                                           y,
//                                                            int32_t z,
//                                                            const
//                                                            block_entity*,
//                                                            void*),
//                                           void* custom_data);
//
// void mc_schem_region_visit_entities(const region*,
//                                     void (*callback)(const entity*, void*),
//                                     void* custom_data);
}

class deleter {
 public:
  static void operator()(block* block) { mc_schem_destroy_block(block); }
  static void operator()(error* err) { mc_schem_destroy_error(err); }
  static void operator()(region* ptr) { mc_schem_destroy_region(ptr); }
};

class block {
 public:
  block() = delete;
  ~block() = delete;
  block(const block&) = delete;
  block(block&&) = delete;
  block& operator=(const block&) = delete;
  block& operator=(block&&) = delete;

  [[nodiscard]] static std::unique_ptr<block, deleter> create() {
    return std::unique_ptr<block, deleter>{mc_schem_create_block()};
  }

  [[nodiscard]] std::string id() const {
    std::string ret;
    rust_string_receiver receiver{ret};
    mc_schem_block_get_id(this, &receiver);
    return ret;
  }

  [[nodiscard]] std::string namespace_() const {
    std::string ret;
    rust_string_receiver receiver{ret};
    mc_schem_block_get_namespace(this, &receiver);
    return ret;
  }

  void set_id(const std::string& id) {
    mc_schem_block_set_id(this, id.c_str());
  }
  void set_namespace(const std::string& ns) {
    mc_schem_block_set_namespace(this, ns.c_str());
  }

  [[nodiscard]] std::string full_id() const {
    std::string ret;
    rust_string_receiver receiver{ret};
    mc_schem_block_get_full_id(this, &receiver);
    return ret;
  }

  [[nodiscard]] std::expected<void, block_id_parse_error> reset(
      const std::string& full_id) {
    block_id_parse_error err{};
    if (mc_schem_block_reset(this, full_id.c_str(), &err)) {
      return {};
    }
    return std::unexpected{err};
  }

  template <class visitor_type>
    requires std::is_invocable_r_v<void, visitor_type, std::string&&,
                                   std::string&&>
  void visit_attributes(visitor_type&& visitor) const {
    auto callback = [](const char8_t* key, size_t key_bytes,
                       const char8_t* value, size_t value_bytes,
                       void* custom_data) {
      std::u8string_view u8key{key, key_bytes}, u8value{value, value_bytes};
      std::string key_str{u8key.begin(), u8key.end()},
          value_str{u8value.begin(), u8value.end()};

      auto& func = *reinterpret_cast<visitor_type*>(custom_data);
      func(std::move(key_str), std::move(value_str));
    };

    mc_schem_block_visit_attributes(this, callback, &visitor);
  }

  void erase_attribute(const std::string& key) {
    mc_schem_block_erase_attribute(this, key.c_str());
  }

  void set_attribute(const std::string& key, const std::string& value) {
    mc_schem_block_set_attribute(this, key.c_str(), value.c_str());
  }

  [[nodiscard]] bool is_air() const { return mc_schem_block_is_air(this); }
  [[nodiscard]] bool is_structure_void() const {
    return mc_schem_block_is_structure_void(this);
  }
};

class error {
 public:
  error() = delete;
  ~error() = delete;
  error(const error&) = delete;
  error(const error&&) = delete;
  error& operator=(const error&) = delete;
  error& operator=(error&&) = delete;

  [[nodiscard]] std::string message() const {
    std::string ret;
    rust_string_receiver receiver{ret};
    mc_schem_error_get_message(this, &receiver);
    return ret;
  }
};

class region {
 public:
  region() = delete;
  ~region() = delete;
  region(const region&) = delete;
  region(region&&) = delete;
  region& operator=(const region&) = delete;
  region& operator=(region&&) = delete;

  [[nodiscard]] static std::unique_ptr<region, deleter> create(
      int32_t shape_x, int32_t shape_y, int32_t shape_z) {
    return std::unique_ptr<region, deleter>{
        mc_schem_create_region(shape_x, shape_y, shape_z)};
  }

  [[nodiscard]] static std::expected<std::unique_ptr<region, deleter>,
                                     std::unique_ptr<error, deleter>>
  create_with_palette(const std::array<int32_t, 3>& shape,
                      const std::span<const block*> palette) {
    const auto [x, y, z] = shape;

    error* err{nullptr};
    auto result = mc_schem_create_region_with_palette(x, y, z, palette.data(),
                                                      palette.size(), &err);
    if (result) {
      assert(err == nullptr);
      return std::unique_ptr<region, deleter>{result};
    }
    assert(err);
    return std::unexpected{std::unique_ptr<error, deleter>{err}};
  }

  [[nodiscard]] std::array<int32_t, 3> offset() const {
    int32_t x{0}, y{0}, z{0};
    mc_schem_region_get_offset(this, &x, &y, &z);
    return {x, y, z};
  }

  [[nodiscard]] std::array<int32_t, 3> size_xyz() const {
    int32_t x{0}, y{0}, z{0};
    mc_schem_region_get_size(this, &x, &y, &z);
    return {x, y, z};
  }

  [[nodiscard]] std::array<int32_t, 3> size_yzx() const {
    const auto [x, y, z] = size_xyz();
    return {y, z, x};
  }

  [[nodiscard]] size_t palette_size() const {
    return mc_schem_region_palette_get_size(this);
  }

  [[nodiscard]] const block* palette(size_t index) const {
    return mc_schem_region_palette_get_block(this, index);
  }

  [[nodiscard]] std::vector<const block*> full_palette() const {
    std::vector<const block*> ret;
    const size_t n = palette_size();
    ret.reserve(n);
    for (size_t i = 0; i < n; i++) {
      ret.emplace_back(palette(i));
    }
    return ret;
  }
};

}  // namespace mc_schem

#endif  // MC_SCHEM_MC_SCHEM_HPP
