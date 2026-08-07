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

#include <cstddef>
#include <cstdint>
#include <expected>
#include <memory>
#include <optional>
#include <string>
#include <string_view>
#include <type_traits>

#include "mc_schem.hpp"

namespace mc_schem {
// Forward declarations
class block;

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
// void mc_schem_destroy_block_id_parse_error(block_id_parse_error* error);

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
}

class deleter {
 public:
  static void operator()(block* block) { mc_schem_destroy_block(block); }
  // static void operator()(block_id_parse_error* error) {
  //   mc_schem_destroy_block_id_parse_error(error);
  // }
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

}  // namespace mc_schem

#endif  // MC_SCHEM_MC_SCHEM_HPP
