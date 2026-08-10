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
#include <format>
#include <memory>
#include <optional>
#include <span>
#include <stdexcept>
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
class meta_data_ir;

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
/// Create region with given palette. If error, error_dest is box of error and
/// returns null; Otherwise error_dest is null.
[[nodiscard]] region* mc_schem_create_region_with_palette(
    int32_t size_x, int32_t size_y, int32_t size_z,
    const block* const palette[], size_t palette_size, error** error_dest);
void mc_schem_destroy_region(region* region);
void mc_schem_destroy_entity(entity* entity);
void mc_schem_destroy_block_entity(block_entity* be);
void mc_schem_destroy_pending_tick(pending_tick* tick);
void mc_schem_destroy_schematic(schematic* schematic);
void mc_schem_destroy_meta_data_ir(meta_data_ir* mdata);

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
////////////////////////////////////////////////////////////////////////////////
// Metainfo
////////////////////////////////////////////////////////////////////////
void mc_schem_region_get_name(const region*, const rust_string_receiver* dest);
void mc_schem_region_get_offset(const region*, int32_t* dest_x, int32_t* dest_y,
                                int32_t* dest_z);
void mc_schem_region_set_name(region*, const char* str);
void mc_schem_region_set_offset(region*, int32_t x, int32_t y, int32_t z);
void mc_schem_region_get_size(const region*, int32_t* size_x, int32_t* size_y,
                              int32_t* size_z);
bool mc_schem_region_is_dense(const region*);
void mc_schem_region_reshape(region*, int32_t x, int32_t y, int32_t z);
// Palette
////////////////////////////////////////////////////////////////////////
size_t mc_schem_region_palette_get_size(const region*);
const block* mc_schem_region_palette_get_block(const region*, size_t index);
/// Add block into palette (deep copy). If identical block already exist in
/// palette, don't copy; otherwise append. Returns index of this block in
/// palette
uint16_t mc_schem_region_find_or_append_to_palette(region* region,
                                                   const block* block);
uint16_t mc_schem_region_find_in_palette(const region*, const block*, bool* ok);
// Entity
////////////////////////////////////////////////////////////////////////
size_t mc_schem_region_get_entities_count(const region*);
const entity* mc_schem_region_get_entity(const region*, size_t index);
entity* mc_schem_region_get_entity_mut(region*, size_t index);
entity* mc_schem_region_erase_entity(region*, size_t index);
/// Clone entity into region, returns index
size_t mc_schem_region_add_entity(region*, const entity*);
// Block entity
////////////////////////////////////////////////////////////////////////
size_t mc_schem_region_get_block_entities_count(const region*);
void mc_schem_region_visit_block_entities(const region*,
                                          void (*callback)(int32_t x, int32_t y,
                                                           int32_t z,
                                                           const block_entity*,
                                                           void*),
                                          void* custom_data);
const block_entity* mc_schem_region_get_block_entity(const region*, int32_t x,
                                                     int32_t y, int32_t z);
block_entity* mc_schem_region_get_block_entity_mut(region*, int32_t x,
                                                   int32_t y, int32_t z);
/// Copy and insert block entity into given coordinate. If previous BE exists,
/// it will be moved out and boxed and returned as ptr. If be is null, erase old
/// value
block_entity* mc_schem_region_add_block_entity(region*, int32_t x, int32_t y,
                                               int32_t z,
                                               const block_entity* nullable);

// Get pending ticks. Currently no rule to add or remove pending ticks. Will do
// this later if pending ticks is found to be useful
////////////////////////////////////////////////////////////////////////
size_t mc_schem_region_get_pending_ticks_count(const region*, int32_t x,
                                               int32_t y, int32_t z);
const pending_tick* mc_schem_region_get_pending_tick(const region*, int32_t x,
                                                     int32_t y, int32_t z,
                                                     size_t idx);
pending_tick* mc_schem_region_get_pending_tick_mut(region*, int32_t x,
                                                   int32_t y, int32_t z,
                                                   size_t idx);
// Get and set blocks
////////////////////////////////////////////////////////////////////////
uint16_t mc_schem_region_get_block_index(const region*, int32_t x, int32_t y,
                                         int32_t z, bool* ok);
// returns ok
bool mc_schem_region_set_block_by_index(region*, int32_t x, int32_t y,
                                        int32_t z, uint16_t idx);
// returns ok
bool mc_schem_region_set_block_by_block(region*, int32_t x, int32_t y,
                                        int32_t z, const block* blk);
// Complex operations for region
////////////////////////////////////////////////////////////////////////
error* mc_schem_region_shrink_palette(region*);
void mc_schem_region_fill_with(region*, const block* blk);
void mc_schem_region_convert_to_sparse(region*);
void mc_schem_region_convert_to_dense(region*);
uint64_t mc_schem_region_total_blocks(const region*, bool include_air);
/// Visit all blocks in region. If explicit-only, skip background blocks for
/// sparse region. Pending ticks is ignored on-purpose, because currently no
/// perfect way to pass &[PendingTick] while pending_tick has incorrect size
/// than rust
void mc_schem_region_visit_blocks(
    const region*, bool explicit_only,
    void (*callback)(int32_t x, int32_t y, int32_t z, uint16_t block_idx,
                     const block*, const block_entity*, void* custom_data),
    void* custom_data);

// Entity
////////////////////////////////////////////////////////////////////////////////

void mc_schem_entity_get_position(const entity*, int32_t* x, int32_t* y,
                                  int32_t* z, double* fp_x, double* fp_y,
                                  double* fp_z);
}

class deleter {
 public:
  static void operator()(block* block) { mc_schem_destroy_block(block); }
  static void operator()(error* err) { mc_schem_destroy_error(err); }
  static void operator()(region* ptr) { mc_schem_destroy_region(ptr); }
  static void operator()(entity* ptr) { mc_schem_destroy_entity(ptr); }
  static void operator()(block_entity* ptr) {
    mc_schem_destroy_block_entity(ptr);
  }
  static void operator()(pending_tick* ptr) {
    mc_schem_destroy_pending_tick(ptr);
  }
  static void operator()(schematic* ptr) { mc_schem_destroy_schematic(ptr); }
  static void operator()(meta_data_ir* ptr) {
    mc_schem_destroy_meta_data_ir(ptr);
  }
};

/// Block for Minecraft.
/// Note: sizeof(block) is fake. Never construct from C/C++, only construct,
/// allocate, destroy and deallocate in Rust. Always use `this` as handle.
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

  [[nodiscard]] std::string id() const& {
    std::string ret;
    rust_string_receiver receiver{ret};
    mc_schem_block_get_id(this, &receiver);
    return ret;
  }

  [[nodiscard]] std::string namespace_() const& {
    std::string ret;
    rust_string_receiver receiver{ret};
    mc_schem_block_get_namespace(this, &receiver);
    return ret;
  }

  void set_id(const std::string& id) & {
    mc_schem_block_set_id(this, id.c_str());
  }
  void set_namespace(const std::string& ns) & {
    mc_schem_block_set_namespace(this, ns.c_str());
  }

  [[nodiscard]] std::string full_id() const& {
    std::string ret;
    rust_string_receiver receiver{ret};
    mc_schem_block_get_full_id(this, &receiver);
    return ret;
  }

  [[nodiscard]] std::expected<void, block_id_parse_error> reset(
      const std::string& full_id) & {
    block_id_parse_error err{};
    if (mc_schem_block_reset(this, full_id.c_str(), &err)) {
      return {};
    }
    return std::unexpected{err};
  }

  template <class visitor_type>
    requires std::is_invocable_r_v<void, visitor_type, std::string&&,
                                   std::string&&>
  void visit_attributes(visitor_type&& visitor) const& {
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

  void erase_attribute(const std::string& key) & {
    mc_schem_block_erase_attribute(this, key.c_str());
  }

  void set_attribute(const std::string& key, const std::string& value) & {
    mc_schem_block_set_attribute(this, key.c_str(), value.c_str());
  }

  [[nodiscard]] bool is_air() const& { return mc_schem_block_is_air(this); }
  [[nodiscard]] bool is_structure_void() const& {
    return mc_schem_block_is_structure_void(this);
  }
};

/// Error from Rust side
/// Note: sizeof is fake. Never construct from C/C++, only construct,
/// allocate, destroy and deallocate in Rust. Always use `this` as handle.
class error {
 public:
  error() = delete;
  ~error() = delete;
  error(const error&) = delete;
  error(const error&&) = delete;
  error& operator=(const error&) = delete;
  error& operator=(error&&) = delete;

  [[nodiscard]] std::string message() const& {
    std::string ret;
    rust_string_receiver receiver{ret};
    mc_schem_error_get_message(this, &receiver);
    return ret;
  }
};

/// Region of schematic
/// Note: sizeof(region) is fake. Never construct from C/C++, only construct,
/// allocate, destroy and deallocate in Rust. Always use `this` as handle.
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

  [[nodiscard]] std::string name() const& {
    std::string ret;
    rust_string_receiver receiver{ret};
    mc_schem_region_get_name(this, &receiver);
    return ret;
  }

  [[nodiscard]] std::array<int32_t, 3> offset() const& {
    int32_t x{0}, y{0}, z{0};
    mc_schem_region_get_offset(this, &x, &y, &z);
    return {x, y, z};
  }

  void set_name(const std::string& name) & {
    mc_schem_region_set_name(this, name.c_str());
  }

  void set_offset(const std::array<int32_t, 3>& offset) & {
    mc_schem_region_set_offset(this, offset[0], offset[1], offset[2]);
  }

  [[nodiscard]] std::array<int32_t, 3> size_xyz() const& {
    int32_t x{0}, y{0}, z{0};
    mc_schem_region_get_size(this, &x, &y, &z);
    return {x, y, z};
  }

  [[nodiscard]] std::array<int32_t, 3> size_yzx() const& {
    const auto [x, y, z] = size_xyz();
    return {y, z, x};
  }

  [[nodiscard]] bool is_dense() const& {
    return mc_schem_region_is_dense(this);
  }
  [[nodiscard]] bool is_sparse() const& { return not is_dense(); }

  void reshape(const std::array<int32_t, 3>& shape) & {
    mc_schem_region_reshape(this, shape[0], shape[1], shape[2]);
  }

  [[nodiscard]] size_t palette_size() const& {
    return mc_schem_region_palette_get_size(this);
  }

  [[nodiscard]] const block* palette(size_t index) const& {
    return mc_schem_region_palette_get_block(this, index);
  }

  [[nodiscard]] uint16_t find_or_append_to_palette(const block& blk) & {
    return mc_schem_region_find_or_append_to_palette(this, &blk);
  }

  [[nodiscard]] std::optional<uint16_t> find_in_palette(
      const block& blk) const& {
    bool ok = false;
    const auto result = mc_schem_region_find_in_palette(this, &blk, &ok);
    if (not ok) {
      return std::nullopt;
    }
    assert(result < palette_size());
    return result;
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

  [[nodiscard]] size_t entities_count() const& {
    return mc_schem_region_get_entities_count(this);
  }
  [[nodiscard]] const entity* get_entity(size_t index) const& {
    return mc_schem_region_get_entity(this, index);
  }
  [[nodiscard]] entity* get_entity(size_t index) & {
    return mc_schem_region_get_entity_mut(this, index);
  }
  std::unique_ptr<entity, deleter> erase_entity(size_t index) & {
    auto ptr = mc_schem_region_erase_entity(this, index);
    return std::unique_ptr<entity, deleter>{ptr};
  }
  [[nodiscard]] size_t add_entity(const entity& entity) & {
    return mc_schem_region_add_entity(this, &entity);
  }

  [[nodiscard]] size_t block_entities_count() const& {
    return mc_schem_region_get_block_entities_count(this);
  }
  template <class visitor_type>
    requires std::is_invocable_r_v<void, visitor_type, int32_t, int32_t,
                                   int32_t, const block_entity&>
  void visit_block_entities(visitor_type&& visitor) const& {
    auto func = [](int32_t x, int32_t y, int32_t z, const block_entity* e,
                   void* custom_data) {
      auto& visitor = *reinterpret_cast<visitor_type*>(custom_data);
      visitor(x, y, z, *e);
    };
    mc_schem_region_visit_block_entities(this, func, &visitor);
  }
  [[nodiscard]] const block_entity* get_block_entity(
      const std::array<int32_t, 3>& pos) const& {
    return mc_schem_region_get_block_entity(this, pos[0], pos[1], pos[2]);
  }
  [[nodiscard]] block_entity* get_block_entity(
      const std::array<int32_t, 3>& pos) & {
    return mc_schem_region_get_block_entity_mut(this, pos[0], pos[1], pos[2]);
  }
  /// Insert new, returns previous value (if exist)
  std::unique_ptr<block_entity, deleter> add_block_entity(
      const std::array<int32_t, 3>& pos, const block_entity& e) & {
    auto ret =
        mc_schem_region_add_block_entity(this, pos[0], pos[1], pos[2], &e);
    return std::unique_ptr<block_entity, deleter>{ret};
  }
  /// Returns previous value (if exist)
  std::unique_ptr<block_entity, deleter> erase_block_entity(
      const std::array<int32_t, 3>& pos) & {
    auto ret =
        mc_schem_region_add_block_entity(this, pos[0], pos[1], pos[2], nullptr);
    return std::unique_ptr<block_entity, deleter>{ret};
  }

  [[nodiscard]] size_t pending_ticks_count_at(
      const std::array<int32_t, 3>& pos) const& {
    return mc_schem_region_get_pending_ticks_count(this, pos[0], pos[1],
                                                   pos[2]);
  }

  [[nodiscard]] std::vector<std::reference_wrapper<const pending_tick>>
  pending_ticks_at(const std::array<int32_t, 3>& pos) const& {
    std::vector<std::reference_wrapper<const pending_tick>> ret;
    const size_t n = pending_ticks_count_at(pos);
    ret.reserve(n);
    for (size_t i = 0; i < n; i++) {
      auto ptr =
          mc_schem_region_get_pending_tick(this, pos[0], pos[1], pos[2], i);
      ret.emplace_back(std::ref(*ptr));
    }
    return ret;
  }
  [[nodiscard]] std::vector<std::reference_wrapper<pending_tick>>
  pending_ticks_at(const std::array<int32_t, 3>& pos) & {
    std::vector<std::reference_wrapper<pending_tick>> ret;
    const size_t n = pending_ticks_count_at(pos);
    ret.reserve(n);
    for (size_t i = 0; i < n; i++) {
      auto ptr =
          mc_schem_region_get_pending_tick_mut(this, pos[0], pos[1], pos[2], i);
      ret.emplace_back(std::ref(*ptr));
    }
    return ret;
  }

  [[nodiscard]] std::optional<uint16_t> block_index_at(
      const std::array<int32_t, 3>& pos) const& {
    bool ok = false;
    const uint16_t ret =
        mc_schem_region_get_block_index(this, pos[0], pos[1], pos[2], &ok);
    if (not ok) {
      return std::nullopt;
    }
    assert(ret < this->palette_size());
    return ret;
  }

  [[nodiscard]] bool contains_coordinate(
      const std::array<int32_t, 3>& pos) const& {
    return this->block_index_at(pos).has_value();
  }

  [[nodiscard]] std::optional<
      std::tuple<uint16_t, std::reference_wrapper<const block>>>
  block_at(const std::array<int32_t, 3>& pos) const& {
    const auto idx_opt = this->block_index_at(pos);
    if (not idx_opt) {
      return std::nullopt;
    }
    const uint16_t idx = idx_opt.value();
    auto blkp = this->palette(idx);
    assert(blkp);
    return std::make_tuple(idx, std::ref(*blkp));
  }

  [[nodiscard]] std::optional<std::tuple<
      uint16_t, std::reference_wrapper<const block>, const block_entity*,
      std::vector<std::reference_wrapper<const pending_tick>>>>
  block_info_at(const std::array<int32_t, 3>& pos) const& {
    const auto blk_opt = this->block_at(pos);
    if (not blk_opt) {
      return std::nullopt;
    }
    const auto [idx, blk] = blk_opt.value();

    const auto bep = this->get_block_entity(pos);
    auto pts = this->pending_ticks_at(pos);
    return std::make_tuple(idx, blk, bep, std::move(pts));
  }

  void set_block(const std::array<int32_t, 3>& pos, uint16_t blkid) & {
    if (not mc_schem_region_set_block_by_index(this, pos[0], pos[1], pos[2],
                                               blkid)) {
      // This error is rare. Usually don't process with exception
      throw std::runtime_error{
          std::format("Unable to set block ({}, {}, {}) to index {}. Block out "
                      "of range or block id out of range",
                      pos[0], pos[1], pos[2], blkid)};
    }
  }
  void set_block(const std::array<int32_t, 3>& pos, const block& blk) & {
    if (not mc_schem_region_set_block_by_block(this, pos[0], pos[1], pos[2],
                                               &blk)) {
      // This error is rare. Usually don't process with exception
      throw std::runtime_error{
          std::format("Unable to set block ({}, {}, {}). Block out of range or "
                      "palette size out of range",
                      pos[0], pos[1], pos[2])};
    }
  }

  std::expected<void, std::unique_ptr<error, deleter>> shrink_palette() & {
    auto err =
        std::unique_ptr<error, deleter>{mc_schem_region_shrink_palette(this)};
    if (err) {
      return std::unexpected{std::move(err)};
    }
    return {};
  }

  void fill_with(const block& blk) & { mc_schem_region_fill_with(this, &blk); }
  void convert_to_sparse() & { mc_schem_region_convert_to_sparse(this); }
  void convert_to_dense() & { mc_schem_region_convert_to_dense(this); }

  template <class visitor_type>
    requires std::is_invocable_r_v<void, visitor_type, std::array<int32_t, 3>,
                                   uint16_t, const block&, const block_entity*>
  void visit_blocks(visitor_type&& visitor, bool explicit_only) const& {
    auto func = [](int32_t x, int32_t y, int32_t z, uint16_t blkid,
                   const block* blkp, const block_entity* be,
                   void* custom_data) {
      auto& vis = *reinterpret_cast<visitor_type*>(custom_data);
      const std::array<int32_t, 3> pos{x, y, z};
      vis(pos, blkid, *blkp, be);
    };
    mc_schem_region_visit_blocks(this, explicit_only, func, &visitor);
  }
};

class entity {
 public:
  entity() = delete;
  entity(const entity&) = delete;
  entity(entity&&) = delete;
  entity& operator=(const entity&) = delete;
  entity& operator=(entity&&) = delete;
  ~entity() = delete;

  [[nodiscard]] std::pair<std::array<int32_t, 3>, std::array<double, 3>>
  position() const& {
    std::array<int32_t, 3> block_pos{0, 0, 0};
    std::array<double, 3> pos{0, 0, 0};
    mc_schem_entity_get_position(this, &block_pos[0], &block_pos[1],
                                 &block_pos[2], &pos[0], &pos[1], &pos[2]);
    return std::make_pair(block_pos, pos);
  }
};

}  // namespace mc_schem

#endif  // MC_SCHEM_MC_SCHEM_HPP
