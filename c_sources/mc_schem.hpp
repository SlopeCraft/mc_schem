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
#include <istream>
#include <memory>
#include <optional>
#include <ostream>
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

enum class common_block : uint16_t {
  air = 0,
  structure_void = 1,
};

// Forward declarations
class block;
class error;
class entity;
class block_entity;
class pending_tick;
class region;
class metadata_ir;
class schematic;
class nbt_hashmap;

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

namespace internal {
inline void save_error_info(std::string_view msg, char* error_message_dest,
                            size_t error_message_capacity) {
  const size_t len = std::min(msg.size(), error_message_capacity);
  std::copy_n(msg.data(), len, error_message_dest);
}
}  // namespace internal

struct rust_reader {
  size_t (*func_read)(uint8_t* dest, size_t dest_capacity, bool* ok_nonnull,
                      char* error_message_dest, size_t error_message_capacity,
                      void* custom_data){nullptr};
  void* custom_data{nullptr};

  explicit rust_reader(std::istream& is) : custom_data{&is} {
    func_read = [](uint8_t* dest, size_t dest_capacity, bool* ok,
                   char* error_message_dest, size_t error_message_capacity,
                   void* custom_data) -> size_t {
      auto& is = *reinterpret_cast<std::istream*>(custom_data);

      try {
        const size_t read_bytes =
            is.readsome(reinterpret_cast<char*>(dest),
                        static_cast<std::streamsize>(dest_capacity));
        *ok = is.good() or is.eof();
        return read_bytes;
      } catch (const std::exception& e) {
        *ok = false;
        internal::save_error_info(e.what(), error_message_dest,
                                  error_message_capacity);
        return 0;
      } catch (...) {
        *ok = false;
        internal::save_error_info("unknown error", error_message_dest,
                                  error_message_capacity);
        return 0;
      }
    };
  }
};

struct rust_writer {
  size_t (*func_write)(const uint8_t* buf, size_t buf_bytes, bool* ok_nonnull,
                       char* error_message_dest, size_t error_message_capacity,
                       void* custom_data){nullptr};
  bool (*func_flush)(void* custom_data, char* error_message_dest,
                     size_t error_message_capacity){nullptr};
  void* custom_data{nullptr};

  explicit rust_writer(std::ostream& os) : custom_data{&os} {
    func_write = [](const uint8_t* buf, size_t buf_bytes, bool* ok,
                    char* error_message_dest, size_t error_message_capacity,
                    void* handle) -> size_t {
      auto& os = *static_cast<std::ostream*>(handle);

      try {
        os.write(reinterpret_cast<const char*>(buf),
                 static_cast<std::streamsize>(buf_bytes));
        *ok = os.good() or os.eof();
        return buf_bytes;
      } catch (const std::exception& e) {
        internal::save_error_info(e.what(), error_message_dest,
                                  error_message_capacity);
      } catch (...) {
        internal::save_error_info("unknown error", error_message_dest,
                                  error_message_capacity);
      }
      *ok = false;
      return 0;
    };
    func_flush = [](void* handle, char* error_message_dest,
                    size_t error_message_capacity) {
      auto& os = *static_cast<std::ostream*>(handle);
      try {
        os.flush();
        return os.good() or os.eof();
      } catch (const std::exception& e) {
        internal::save_error_info(e.what(), error_message_dest,
                                  error_message_capacity);
      } catch (...) {
        internal::save_error_info("unknown error", error_message_dest,
                                  error_message_capacity);
      }
      return false;
    };
  }

  explicit rust_writer(std::vector<uint8_t>& vec) : custom_data{&vec} {
    func_write = [](const uint8_t* buf, size_t buf_bytes, bool* ok,
                    char* error_message_dest, size_t error_message_capacity,
                    void* handle) -> size_t {
      auto& dest = *static_cast<std::vector<uint8_t>*>(handle);
      try {
        dest.append_range(std::span{buf, buf_bytes});
        *ok = true;
        return buf_bytes;
      } catch (const std::exception& e) {
        // most possible: out of memory
        *ok = false;
        internal::save_error_info(e.what(), error_message_dest,
                                  error_message_capacity);
        return 0;
      }
      // std::vector is impossible to throw non-standard exceptions
    };
    func_flush = [](void* handle [[maybe_unused]],
                    char* error_message_dest [[maybe_unused]],
                    size_t error_message_capacity [[maybe_unused]]) -> bool {
      // nothing to do. Vector don't need to flush
      return true;
    };
  }
};

struct vanilla_structure_load_option {
  common_block background_block{common_block::structure_void};
};
struct vanilla_structure_save_option {
  uint32_t compress_level{9};
  bool keep_air{true};
};
struct litematica_load_option {};
struct litematica_save_option {
  uint32_t compress_level{9};
  bool rename_duplicated_regions{true};
};
struct world_edit13_load_option {};
struct world_edit13_save_option {
  uint32_t compress_level{9};
  common_block background_block{common_block::air};
};
struct world_edit12_load_option {
  /// Data version of this schematic. Data version is not stored in
  /// `.schematic`, so we should assign it.
  int32_t data_version{1343};  // Java_1_12_2
};

extern "C" {
// destroy
void mc_schem_destroy_block(block* block);
void mc_schem_destroy_error(error*);
void mc_schem_destroy_region(region* region);
void mc_schem_destroy_entity(entity* entity);
void mc_schem_destroy_block_entity(block_entity* be);
void mc_schem_destroy_pending_tick(pending_tick* tick);
void mc_schem_destroy_metadata_ir(metadata_ir* mdata);
void mc_schem_destroy_schematic(schematic* schematic);
void mc_schem_destroy_nbt_hashmap(nbt_hashmap* hashmap);

// clone
[[nodiscard]] block* mc_schem_clone_block(const block*);
// [[nodiscard]] error* mc_schem_clone_error(const error*);
[[nodiscard]] region* mc_schem_clone_region(const region*);
[[nodiscard]] entity* mc_schem_clone_entity(const entity*);
[[nodiscard]] block_entity* mc_schem_clone_block_entity(const block_entity*);
[[nodiscard]] pending_tick* mc_schem_clone_pending_tick(const pending_tick*);
[[nodiscard]] metadata_ir* mc_schem_clone_metadata_ir(const metadata_ir*);
[[nodiscard]] schematic* mc_schem_clone_schematic(const schematic*);
[[nodiscard]] nbt_hashmap* mc_schem_clone_nbt_hashmap(const nbt_hashmap*);

// swap: deep-swap on data. No memory allocation or release
void mc_schem_swap_block(block* a, block* b);
void mc_schem_swap_region(region* a, region* b);
void mc_schem_swap_entity(entity* a, entity* b);
void mc_schem_swap_block_entity(block_entity* a, block_entity* b);
void mc_schem_swap_pending_tick(pending_tick* a, pending_tick* b);
void mc_schem_swap_metadata_ir(metadata_ir* a, metadata_ir* b);
void mc_schem_swap_nbt_hashmap(nbt_hashmap* a, nbt_hashmap* b);
void mc_schem_swap_schematic(schematic* a, schematic* b);

// NBT
////////////////////////////////////////////////////////////////////////////////
/// Create empty hashmap
[[nodiscard]] nbt_hashmap* mc_schem_create_nbt_hashmap();
/// Create nbt hashmap from uncompressed binary (stored in memory)
[[nodiscard]] nbt_hashmap* mc_schem_create_nbt_hashmap_from_binary(
    const uint8_t* buffer, size_t bytes,
    const rust_string_receiver* error_message_receiver);
/// Create nbt hashmap from uncompressed binary (from istream)
[[nodiscard]] nbt_hashmap* mc_schem_create_nbt_hashmap_from_binary_stream(
    rust_reader* src, const rust_string_receiver* error_message_receiver);
/// Size of this hashmap
[[nodiscard]] size_t mc_schem_nbt_hashmap_get_size(const nbt_hashmap*);
/// Dump nbt hashmap to ostream
bool mc_schem_nbt_hashmap_dump_to_binary_stream(
    const nbt_hashmap*, rust_writer* dest,
    const rust_string_receiver* error_message_receiver);
// Block
////////////////////////////////////////////////////////////////////////////////
[[nodiscard]] block* mc_schem_create_block();
[[nodiscard]] block* mc_schem_create_block_from_common(common_block blk);
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
[[nodiscard]] region* mc_schem_create_region(int32_t size_x, int32_t size_y,
                                             int32_t size_z);
/// Create region with given palette. If error, error_dest is box of error and
/// returns null; Otherwise error_dest is null.
[[nodiscard]] region* mc_schem_create_region_with_palette(
    int32_t size_x, int32_t size_y, int32_t size_z,
    const block* const palette[], size_t palette_size, error** error_dest);
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
/// Move entity (must from box) into region, returns index. new_entity moved and
/// released in this operation.
size_t mc_schem_region_add_entity_move(region*, entity* new_entity);
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
/// Move and insert block entity into given coordinate. If previous BE exists,
/// it will be moved out and boxed and returned as ptr. If be is null, erase old
/// value
block_entity* mc_schem_region_add_block_entity_move(
    region*, int32_t x, int32_t y, int32_t z, block_entity* new_be_nullable);

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
[[nodiscard]] error* mc_schem_region_shrink_palette(region*);
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
const nbt_hashmap* mc_schem_entity_get_tags(const entity*);
nbt_hashmap* mc_schem_entity_get_tags_mut(entity*);
/// Deep copy new_value into entity, returns old value by box (transfer
/// ownership
nbt_hashmap* mc_schem_entity_set_tags(entity*, const nbt_hashmap* new_value);

// Block entity
////////////////////////////////////////////////////////////////////////////////
block_entity* mc_schem_create_block_entity();
const nbt_hashmap* mc_schem_block_entity_get_tags(const block_entity*);
nbt_hashmap* mc_schem_block_entity_get_tags_mut(block_entity*);
/// Deep copy new_value into entity, returns old value by box (transfer
/// ownership
nbt_hashmap* mc_schem_block_entity_set_tags(block_entity*,
                                            const nbt_hashmap* new_value);
// Meta data ir
////////////////////////////////////////////////////////////////////////////////
[[nodiscard]] metadata_ir* mc_schem_create_metadata_ir(
    int32_t data_version, error** dest_err_non_null);

int32_t mc_schem_metadata_ir_get_mc_data_version(const metadata_ir*);
int64_t mc_schem_metadata_ir_get_time_created(const metadata_ir*);
int64_t mc_schem_metadata_ir_get_time_modified(const metadata_ir*);
void mc_schem_metadata_ir_get_author(const metadata_ir*,
                                      const rust_string_receiver* dest);
void mc_schem_metadata_ir_get_name(const metadata_ir*,
                                    const rust_string_receiver* dest);
int32_t mc_schem_metadata_ir_get_litematica_version(const metadata_ir*);
int32_t mc_schem_metadata_ir_get_litematica_subversion(
    const metadata_ir*, bool* dest_exist_non_null);
int32_t mc_schem_metadata_ir_get_schem_version(const metadata_ir*);
void mc_schem_metadata_ir_get_schem_offset(const metadata_ir*,
                                            int32_t* dest_x, int32_t* dest_y,
                                            int32_t* dest_z);
bool mc_schem_metadata_ir_get_schem_we_offset(const metadata_ir*,
                                               int32_t* dest_x, int32_t* dest_y,
                                               int32_t* dest_z);
bool mc_schem_metadata_ir_get_schem_world_edit_version(
    const metadata_ir*, const rust_string_receiver*);
bool mc_schem_metadata_ir_get_schem_editing_platform(
    const metadata_ir*, const rust_string_receiver*);
bool mc_schem_metadata_ir_get_schem_origin(const metadata_ir*,
                                            int32_t* dest_x, int32_t* dest_y,
                                            int32_t* dest_z);
void mc_schem_metadata_ir_get_schem_material(const metadata_ir*,
                                              const rust_string_receiver*);
// Schematic
////////////////////////////////////////////////////////////////////////////////
/// Create and return an empty schematic, with default metadata
[[nodiscard]] schematic* mc_schem_create_schematic();
[[nodiscard]] const metadata_ir* mc_schem_schematic_get_metadata(
    const schematic*);
[[nodiscard]] metadata_ir* mc_schem_schematic_get_metadata_mut(schematic*);
/// Deep copy given meta data, move old value to heap and return
metadata_ir* mc_schem_schematic_set_metadata(schematic*, const metadata_ir*);
[[nodiscard]] size_t mc_schem_schematic_get_regions_count(const schematic*);
[[nodiscard]] const region* mc_schem_schematic_get_region(const schematic*,
                                                          size_t idx);
[[nodiscard]] region* mc_schem_schematic_get_region_mut(schematic*, size_t idx);
/// Remove a region from schematic, move to heap and return its pointer.
/// Transfer ownership.
[[nodiscard]] region* mc_schem_schematic_remove_region(schematic*, size_t idx);
/// Remove all regions from schematic
void mc_schem_schematic_clear_all_regions(schematic*);
/// Deep copy new_region into given index, return its pointer in schematic. If
/// index out of range (for example, insert to index=4 with only 3 regions,
/// nothing will be done and returns nullptr)
region* mc_schem_schematic_insert_region(schematic*, const region* new_region,
                                         size_t index);
/// Move new_region (owning, must from box) into given index, return its pointer
/// (non-owning). If index out of range, do nothing. new_region is moved and
/// released in this operation.
region* mc_schem_schematic_insert_region_move(schematic*, region* new_region,
                                              size_t index);
/// Returns positive value if coordinate hits a region. Otherwise return -1. All
/// negative value should be considered as invalid
/// The word "first" means first hit region. Schematic have multiple regions,
/// usually no overlap is expected. If multiple regions overlaps, blocks in
/// first region will live
[[nodiscard]] ptrdiff_t mc_schem_schematic_get_first_region_index_at(
    const schematic*, int32_t x, int32_t y, int32_t z);
[[nodiscard]] uint16_t mc_schem_schematic_get_first_block_index_at(
    const schematic*, int32_t x, int32_t y, int32_t z, bool* ok_nonnull);
[[nodiscard]] const block* mc_schem_schematic_get_first_block_at(
    const schematic*, int32_t x, int32_t y, int32_t z);
[[nodiscard]] const block_entity* mc_schem_schematic_get_first_block_entity_at(
    const schematic*, int32_t x, int32_t y, int32_t z);
[[nodiscard]] size_t mc_schem_schematic_get_first_pending_ticks_count_at(
    const schematic*, int32_t x, int32_t y, int32_t z);
[[nodiscard]] const pending_tick* mc_schem_schematic_get_first_pending_ticks_at(
    const schematic*, int32_t x, int32_t y, int32_t z,
    size_t pending_tick_index);
void mc_schem_schematic_get_shape(const schematic*, int32_t* x, int32_t* y,
                                  int32_t* z);
[[nodiscard]] uint64_t mc_schem_schematic_get_volume(const schematic*);
[[nodiscard]] uint64_t mc_schem_schematic_get_total_blocks(const schematic*,
                                                           bool include_dir);
/// Merge all regions without changing original schematic
[[nodiscard]] region* mc_schem_schematic_to_single_region(
    const schematic*, const block* background_block);
/// Merge all regions in place
void mc_schem_schematic_merge_regions(schematic*,
                                      const block* background_block);

/// Load litematica (*litematic). Returns schematic if ok. Otherwise returns
/// null and sets error dest
[[nodiscard]] schematic* mc_schem_schematic_load_litematica_from_reader(
    rust_reader* src, const litematica_load_option*,
    error** error_dest_nonnull);
/// Load vanilla structure file (*.nbt)
[[nodiscard]] schematic* mc_schem_schematic_load_vanilla_structure_from_reader(
    rust_reader* src, const vanilla_structure_load_option*,
    error** error_dest_nonnull);
/// Load WorldEdit 1.13+ (*.schem)
[[nodiscard]] schematic* mc_schem_schematic_load_world_edit13_from_reader(
    rust_reader* src, const world_edit13_load_option*,
    error** error_dest_nonnull);
/// Load WorldEdit 1.12 (*.schematic)
[[nodiscard]] schematic* mc_schem_schematic_load_world_edit12_from_reader(
    rust_reader* src, const world_edit12_load_option*,
    error** error_dest_nonnull);

/// Write as litematica (*.litematic). Returns null if ok. Otherwise return an
/// error
[[nodiscard]] error* mc_schem_schematic_save_litematica_to_writer(
    const schematic*, rust_writer* dest, const litematica_save_option*);
[[nodiscard]] error* mc_schem_schematic_save_vanilla_structure_to_writer(
    const schematic*, rust_writer* dest, const vanilla_structure_save_option*);
[[nodiscard]] error* mc_schem_schematic_save_world_edit13_to_writer(
    const schematic*, rust_writer* dest, const world_edit13_save_option*);
/// Load schematic from file, automatically identify format by filename
/// extension
[[nodiscard]] schematic* mc_schem_schematic_load_from_file(
    const char* filename, error** error_dest_nonnull);
/// Save schematic to file, auto identify format by filename extension
[[nodiscard]] error* mc_schem_schematic_save_to_file(const schematic*,
                                                     const char* filename);
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
  static void operator()(metadata_ir* ptr) {
    mc_schem_destroy_metadata_ir(ptr);
  }
  static void operator()(nbt_hashmap* ptr) {
    mc_schem_destroy_nbt_hashmap(ptr);
  }
};

using unique_block = std::unique_ptr<block, deleter>;
using unique_entity = std::unique_ptr<entity, deleter>;
using unique_block_entity = std::unique_ptr<block_entity, deleter>;
using unique_pending_tick = std::unique_ptr<pending_tick, deleter>;
using unique_schematic = std::unique_ptr<schematic, deleter>;
using unique_error = std::unique_ptr<error, deleter>;
using unique_region = std::unique_ptr<region, deleter>;
using unique_metadata_ir = std::unique_ptr<metadata_ir, deleter>;
using unique_nbt_hashmap = std::unique_ptr<nbt_hashmap, deleter>;

/// Hashmap of nbt tags
/// Note: sizeof is fake. Never construct from C/C++, only construct,
/// allocate, destroy and deallocate in Rust. Always use `this` as handle.
class nbt_hashmap {
 public:
  nbt_hashmap() = delete;
  ~nbt_hashmap() = delete;
  nbt_hashmap(const nbt_hashmap&) = delete;
  nbt_hashmap(nbt_hashmap&&) = delete;
  nbt_hashmap& operator=(const nbt_hashmap&) = delete;
  nbt_hashmap& operator=(nbt_hashmap&&) = delete;

  [[nodiscard]] static unique_nbt_hashmap create() {
    return unique_nbt_hashmap{mc_schem_create_nbt_hashmap()};
  }

  [[nodiscard]] static std::expected<unique_nbt_hashmap, std::string>
  create_from_binary(std::span<const uint8_t> buffer) {
    std::string error_msg;
    rust_string_receiver receiver{error_msg};

    auto ptr = mc_schem_create_nbt_hashmap_from_binary(
        buffer.data(), buffer.size(), &receiver);
    if (ptr == nullptr) {
      return std::unexpected(std::move(error_msg));
    }
    return unique_nbt_hashmap{ptr};
  }

  [[nodiscard]] static std::expected<unique_nbt_hashmap, std::string>
  create_from_binary_stream(std::istream& is) {
    std::string error_msg;
    rust_string_receiver receiver{error_msg};

    rust_reader isw{is};

    auto ptr = mc_schem_create_nbt_hashmap_from_binary_stream(&isw, &receiver);
    if (ptr == nullptr) {
      return std::unexpected(std::move(error_msg));
    }
    return unique_nbt_hashmap{ptr};
  }

  void swap(nbt_hashmap& another) & {
    mc_schem_swap_nbt_hashmap(this, &another);
  }

  [[nodiscard]] size_t size() const& {
    return mc_schem_nbt_hashmap_get_size(this);
  }

  [[nodiscard]] std::expected<void, std::string> dump_to_stream(
      std::ostream& os) const& {
    std::string error_msg;
    rust_string_receiver receiver{error_msg};
    rust_writer osw{os};
    const bool ok =
        mc_schem_nbt_hashmap_dump_to_binary_stream(this, &osw, &receiver);
    if (not ok) {
      return std::unexpected(std::move(error_msg));
    }
    return {};
  }

  void dump_to_vector(std::vector<uint8_t>& dest) const& {
    dest.clear();
    rust_writer osw{dest};
    std::string error_msg;
    rust_string_receiver receiver{error_msg};
    const bool ok =
        mc_schem_nbt_hashmap_dump_to_binary_stream(this, &osw, &receiver);
    if (not ok) {
      throw std::runtime_error{error_msg};
    }
  }

  [[nodiscard]] std::vector<uint8_t> dump() const& {
    std::vector<uint8_t> result;
    dump_to_vector(result);
    return result;
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

  [[nodiscard]] static unique_block create() {
    return unique_block{mc_schem_create_block()};
  }

  [[nodiscard]] static unique_block from_common(common_block cb) {
    return unique_block{mc_schem_create_block_from_common(cb)};
  }

  [[nodiscard]] auto clone() const& {
    return unique_block{mc_schem_clone_block(this)};
  }

  void swap(block& another) & { mc_schem_swap_block(this, &another); }

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
/// Note: sizeof is fake. Never construct from C/C++, only construct,
/// allocate, destroy and deallocate in Rust. Always use `this` as handle.
class region {
 public:
  region() = delete;
  ~region() = delete;
  region(const region&) = delete;
  region(region&&) = delete;
  region& operator=(const region&) = delete;
  region& operator=(region&&) = delete;

  [[nodiscard]] static unique_region create(int32_t shape_x, int32_t shape_y,
                                            int32_t shape_z) {
    return unique_region{mc_schem_create_region(shape_x, shape_y, shape_z)};
  }

  [[nodiscard]] static std::expected<unique_region, unique_error>
  create_with_palette(const std::array<int32_t, 3>& shape,
                      const std::span<const block* const> palette) {
    const auto [x, y, z] = shape;

    error* err{nullptr};
    auto result = mc_schem_create_region_with_palette(x, y, z, palette.data(),
                                                      palette.size(), &err);
    if (result) {
      assert(err == nullptr);
      return unique_region{result};
    }
    assert(err);
    return std::unexpected{unique_error{err}};
  }

  [[nodiscard]] auto clone() const& {
    return unique_region{mc_schem_clone_region(this)};
  }

  void swap(region& another) & { mc_schem_swap_region(this, &another); }

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
  unique_entity erase_entity(size_t index) & {
    auto ptr = mc_schem_region_erase_entity(this, index);
    return unique_entity{ptr};
  }
  size_t add_entity(const entity& entity) & {
    return mc_schem_region_add_entity(this, &entity);
  }
  size_t add_entity(unique_entity entity) & {
    return mc_schem_region_add_entity_move(this, entity.release());
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
  unique_block_entity add_block_entity(const std::array<int32_t, 3>& pos,
                                       const block_entity& e) & {
    auto ret =
        mc_schem_region_add_block_entity(this, pos[0], pos[1], pos[2], &e);
    return unique_block_entity{ret};
  }
  /// Insert new (by moving), returns previous value (if exist)
  unique_block_entity add_block_entity(const std::array<int32_t, 3>& pos,
                                       unique_block_entity new_be) & {
    auto ret = mc_schem_region_add_block_entity_move(this, pos[0], pos[1],
                                                     pos[2], new_be.release());
    return unique_block_entity{ret};
  }
  /// Returns previous value (if exist)
  unique_block_entity erase_block_entity(const std::array<int32_t, 3>& pos) & {
    auto ret =
        mc_schem_region_add_block_entity(this, pos[0], pos[1], pos[2], nullptr);
    return unique_block_entity{ret};
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

  std::expected<void, unique_error> shrink_palette() & {
    auto err = unique_error{mc_schem_region_shrink_palette(this)};
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

  [[nodiscard]] uint64_t total_blocks(bool include_air) const& {
    return mc_schem_region_total_blocks(this, include_air);
  }
};
/// Entity in Minecraft
/// Note: sizeof is fake. Never construct from C/C++, only construct,
/// allocate, destroy and deallocate in Rust. Always use `this` as handle.
class entity {
 public:
  entity() = delete;
  entity(const entity&) = delete;
  entity(entity&&) = delete;
  entity& operator=(const entity&) = delete;
  entity& operator=(entity&&) = delete;
  ~entity() = delete;

  [[nodiscard]] auto clone() const& {
    return unique_entity{mc_schem_clone_entity(this)};
  }

  void swap(entity& another) & { mc_schem_swap_entity(this, &another); }

  [[nodiscard]] std::pair<std::array<int32_t, 3>, std::array<double, 3>>
  position() const& {
    std::array<int32_t, 3> block_pos{0, 0, 0};
    std::array<double, 3> pos{0, 0, 0};
    mc_schem_entity_get_position(this, &block_pos[0], &block_pos[1],
                                 &block_pos[2], &pos[0], &pos[1], &pos[2]);
    return std::make_pair(block_pos, pos);
  }

  [[nodiscard]] const nbt_hashmap* tags() const& {
    return mc_schem_entity_get_tags(this);
  }
  [[nodiscard]] nbt_hashmap* tags() & {
    return mc_schem_entity_get_tags_mut(this);
  }
  /// Deep copy src, move old value onto heap and returns (transfer ownership)
  unique_nbt_hashmap set_tags(const nbt_hashmap& src) & {
    auto old_value = mc_schem_entity_set_tags(this, &src);
    return unique_nbt_hashmap{old_value};
  }
  /// Move new_value into entity, move old value to box and return it.
  /// Expected usage: set_tags(std::move(new-value-in-unique))
  unique_nbt_hashmap set_tags(unique_nbt_hashmap new_value) & {
    assert(new_value);
    this->tags()->swap(*new_value);
    return new_value;
  }
};
/// Block entity in Minecraft (also known as tile entity)
/// Note: sizeof is fake. Never construct from C/C++, only construct,
/// allocate, destroy and deallocate in Rust. Always use `this` as handle.
class block_entity {
 public:
  block_entity() = delete;
  block_entity(const block_entity&) = delete;
  block_entity(block_entity&&) = delete;
  block_entity& operator=(const block_entity&) = delete;
  block_entity& operator=(block_entity&&) = delete;
  ~block_entity() = delete;

  [[nodiscard]] static unique_block_entity create() {
    return unique_block_entity{mc_schem_create_block_entity()};
  }

  [[nodiscard]] unique_block_entity clone() const& {
    return unique_block_entity{mc_schem_clone_block_entity(this)};
  }

  void swap(block_entity& another) & {
    mc_schem_swap_block_entity(this, &another);
  }

  [[nodiscard]] const nbt_hashmap* tags() const& {
    return mc_schem_block_entity_get_tags(this);
  }
  [[nodiscard]] nbt_hashmap* tags() & {
    return mc_schem_block_entity_get_tags_mut(this);
  }
  unique_nbt_hashmap set_tags(const nbt_hashmap& new_value) & {
    auto old_value = mc_schem_block_entity_set_tags(this, &new_value);
    return unique_nbt_hashmap{old_value};
  }
  /// Move new_value into entity, move old value to box and return it.
  /// Expected usage: set_tags(std::move(new-value-in-unique))
  unique_nbt_hashmap set_tags(unique_nbt_hashmap new_value) & {
    assert(new_value);
    this->tags()->swap(*new_value);
    return new_value;
  }
};
/// Intermediate representation of schematic meta data
/// Note: sizeof is fake. Never construct from C/C++, only construct,
/// allocate, destroy and deallocate in Rust. Always use `this` as handle.
class metadata_ir {
 public:
  metadata_ir() = delete;
  metadata_ir(const metadata_ir&) = delete;
  metadata_ir(metadata_ir&&) = delete;
  metadata_ir& operator=(const metadata_ir&) = delete;
  metadata_ir& operator=(metadata_ir&&) = delete;
  ~metadata_ir() = delete;

  [[nodiscard]] static std::expected<unique_metadata_ir, unique_error> create(
      int32_t data_version) {
    error* dest_err = nullptr;
    if (auto result = mc_schem_create_metadata_ir(data_version, &dest_err)) {
      assert(dest_err == nullptr);
      return unique_metadata_ir{result};
    }
    assert(dest_err);
    return std::unexpected{unique_error{dest_err}};
  }

  [[nodiscard]] auto clone() const& {
    return unique_metadata_ir{mc_schem_clone_metadata_ir(this)};
  }

  void swap(metadata_ir& another) & {
    mc_schem_swap_metadata_ir(this, &another);
  }

  [[nodiscard]] int32_t mc_data_version() const& {
    return mc_schem_metadata_ir_get_mc_data_version(this);
  }
  [[nodiscard]] int64_t time_created() const& {
    return mc_schem_metadata_ir_get_time_created(this);
  }
  [[nodiscard]] int64_t time_modified() const& {
    return mc_schem_metadata_ir_get_time_modified(this);
  }
  [[nodiscard]] std::string author() const& {
    std::string ret;
    rust_string_receiver rsr{ret};
    mc_schem_metadata_ir_get_author(this, &rsr);
    return ret;
  }
  [[nodiscard]] std::string name() const& {
    std::string ret;
    rust_string_receiver rsr{ret};
    mc_schem_metadata_ir_get_name(this, &rsr);
    return ret;
  }
  [[nodiscard]] int32_t litematica_version() const& {
    return mc_schem_metadata_ir_get_litematica_version(this);
  }
  [[nodiscard]] std::optional<int32_t> litematica_subversion() const& {
    bool exist{false};
    const auto ret =
        mc_schem_metadata_ir_get_litematica_subversion(this, &exist);
    if (exist) return ret;
    return std::nullopt;
  }
  [[nodiscard]] int32_t schem_version() const& {
    return mc_schem_metadata_ir_get_schem_version(this);
  }
  [[nodiscard]] std::array<int32_t, 3> schem_offset() const& {
    int32_t x, y, z;
    mc_schem_metadata_ir_get_schem_offset(this, &x, &y, &z);
    return {x, y, z};
  }
  [[nodiscard]] std::optional<std::array<int32_t, 3>> schem_we_offset() const& {
    int32_t x, y, z;
    const bool exist =
        mc_schem_metadata_ir_get_schem_we_offset(this, &x, &y, &z);
    if (exist) return std::array<int32_t, 3>{x, y, z};
    return std::nullopt;
  }
  [[nodiscard]] std::optional<std::string> schem_world_edit_version() const& {
    std::string ret;
    rust_string_receiver rsr{ret};
    const bool exist =
        mc_schem_metadata_ir_get_schem_world_edit_version(this, &rsr);
    if (exist) return ret;
    return std::nullopt;
  }
  [[nodiscard]] std::optional<std::array<int32_t, 3>> schem_origin() const& {
    int32_t x, y, z;
    const bool exist = mc_schem_metadata_ir_get_schem_origin(this, &x, &y, &z);
    if (exist) return std::array<int32_t, 3>{x, y, z};
    return std::nullopt;
  }
  [[nodiscard]] std::string schem_material() const& {
    std::string ret;
    rust_string_receiver rsr{ret};
    mc_schem_metadata_ir_get_schem_material(this, &rsr);
    return ret;
  }
  [[nodiscard]] std::optional<std::string> schem_editing_platform() const& {
    std::string ret;
    rust_string_receiver rsr{ret};
    const bool exist =
        mc_schem_metadata_ir_get_schem_editing_platform(this, &rsr);
    if (exist) return ret;
    return std::nullopt;
  }
};

/// Schematic, a part of minecraft world
/// Note: sizeof is fake. Never construct from C/C++, only construct,
/// allocate, destroy and deallocate in Rust. Always use `this` as handle.
class schematic {
 public:
  schematic() = delete;
  ~schematic() = delete;
  schematic(const schematic&) = delete;
  schematic(schematic&&) = delete;
  schematic& operator=(const schematic&) = delete;
  schematic& operator=(schematic&&) = delete;

  [[nodiscard]] static unique_schematic create() {
    return unique_schematic{mc_schem_create_schematic()};
  }

  [[nodiscard]] unique_schematic clone() const& {
    return unique_schematic{mc_schem_clone_schematic(this)};
  }

  void swap(schematic& another) & { mc_schem_swap_schematic(this, &another); }

  [[nodiscard]] const metadata_ir* metadata() const& {
    return mc_schem_schematic_get_metadata(this);
  }
  [[nodiscard]] metadata_ir* metadata() & {
    return mc_schem_schematic_get_metadata_mut(this);
  }
  /// Deep copy given new value, move previous value onto heap and return
  unique_metadata_ir set_metadata(const metadata_ir& new_metadata) & {
    auto previous_value =
        mc_schem_schematic_set_metadata(this, &new_metadata);
    return unique_metadata_ir{previous_value};
  }
  unique_metadata_ir set_metadata(unique_metadata_ir new_metadata) & {
    new_metadata->swap(*this->metadata());
    return new_metadata;
  }

  [[nodiscard]] size_t regions_count() const& {
    return mc_schem_schematic_get_regions_count(this);
  }
  [[nodiscard]] const region* get_region(size_t region_index) const& {
    return mc_schem_schematic_get_region(this, region_index);
  }
  [[nodiscard]] region* get_region(size_t region_index) & {
    return mc_schem_schematic_get_region_mut(this, region_index);
  }
  unique_region remove_region(size_t region_index) & {
    return unique_region{mc_schem_schematic_remove_region(this, region_index)};
  }
  void clear_all_regions() & { mc_schem_schematic_clear_all_regions(this); }
  /// Deep copy new region and insert into schematic. Returns mut reference to
  /// inserted region. new_region is not modified.
  region* insert_region(const region* new_r, size_t region_index) & {
    return mc_schem_schematic_insert_region(this, new_r, region_index);
  }
  region* append_region(const region* new_r) & {
    return mc_schem_schematic_insert_region(this, new_r, this->regions_count());
  }
  /// Inset new_region into schematic by moving. new_region will be release,
  /// mustn't use after this operation. Returns mut reference to inserted
  /// region.
  region* insert_region(unique_region new_r, size_t region_index) & {
    auto ret = mc_schem_schematic_insert_region_move(this, new_r.release(),
                                                     region_index);
    return ret;
  }
  region* append_region(unique_region new_r) & {
    return this->insert_region(std::move(new_r), this->regions_count());
  }

  [[nodiscard]] std::optional<size_t> first_region_index_at(
      const std::array<int32_t, 3>& pos) const& {
    const ptrdiff_t ret = mc_schem_schematic_get_first_region_index_at(
        this, pos[0], pos[1], pos[2]);
    if (ret >= 0) {
      return ret;
    }
    return std::nullopt;
  }
  [[nodiscard]] std::optional<uint16_t> first_block_index_at(
      const std::array<int32_t, 3>& pos) const& {
    bool ok = false;
    const auto ret = mc_schem_schematic_get_first_block_index_at(
        this, pos[0], pos[1], pos[2], &ok);
    if (not ok) {
      return std::nullopt;
    }
    return ret;
  }
  [[nodiscard]] const block* first_block_at(
      const std::array<int32_t, 3>& pos) const& {
    return mc_schem_schematic_get_first_block_at(this, pos[0], pos[1], pos[2]);
  }
  [[nodiscard]] const block_entity* first_block_entity_at(
      const std::array<int32_t, 3>& pos) const& {
    return mc_schem_schematic_get_first_block_entity_at(this, pos[0], pos[1],
                                                        pos[2]);
  }
  [[nodiscard]] std::vector<const pending_tick*> first_pending_ticks_at(
      const std::array<int32_t, 3>& pos) const& {
    const size_t num = mc_schem_schematic_get_first_pending_ticks_count_at(
        this, pos[0], pos[1], pos[2]);
    std::vector<const pending_tick*> ret;
    ret.reserve(num);
    for (size_t pti = 0; pti < num; ++pti) {
      ret.emplace_back(mc_schem_schematic_get_first_pending_ticks_at(
          this, pos[0], pos[1], pos[2], pti));
      assert(ret.back() not_eq nullptr);
    }
    return ret;
  }

  [[nodiscard]] std::array<int32_t, 3> shape() const& {
    int32_t x = -1, y = -1, z = -1;
    mc_schem_schematic_get_shape(this, &x, &y, &z);
    return {x, y, z};
  }
  [[nodiscard]] uint64_t volume() const& {
    return mc_schem_schematic_get_volume(this);
  }
  [[nodiscard]] uint64_t total_blocks(bool include_air) const& {
    return mc_schem_schematic_get_total_blocks(this, include_air);
  }
  /// Merge all regions without changing original schematic
  [[nodiscard]] unique_region to_single_region(
      const block& background_block) const& {
    return unique_region{
        mc_schem_schematic_to_single_region(this, &background_block)};
  }
  /// Merge all regions in place
  void merge_regions(const block& background_block) & {
    mc_schem_schematic_merge_regions(this, &background_block);
  }
  /// Load schematic from litematica (*.litematic)
  [[nodiscard]] static std::expected<unique_schematic, unique_error>
  load_litematica(std::istream& is, const litematica_load_option& opt) {
    error* err = nullptr;
    rust_reader isw{is};
    auto ret = mc_schem_schematic_load_litematica_from_reader(&isw, &opt, &err);
    if (ret) {
      assert(err == nullptr);
      return unique_schematic{ret};
    }
    assert(err not_eq nullptr);
    return std::unexpected(unique_error{err});
  }
  /// Load schematic from vanilla structure (*.nbt)
  [[nodiscard]] static std::expected<unique_schematic, unique_error>
  load_vanilla_structure(std::istream& is,
                         const vanilla_structure_load_option& opt) {
    error* err = nullptr;
    rust_reader isw{is};
    auto ret =
        mc_schem_schematic_load_vanilla_structure_from_reader(&isw, &opt, &err);
    if (ret) {
      assert(err == nullptr);
      return unique_schematic{ret};
    }
    assert(err not_eq nullptr);
    return std::unexpected(unique_error{err});
  }
  /// Load schematic from world edit 1.13+ (*.schem)
  [[nodiscard]] static std::expected<unique_schematic, unique_error>
  load_world_edit13(std::istream& is, const world_edit13_load_option& opt) {
    error* err = nullptr;
    rust_reader isw{is};
    auto ret =
        mc_schem_schematic_load_world_edit13_from_reader(&isw, &opt, &err);
    if (ret) {
      assert(err == nullptr);
      return unique_schematic{ret};
    }
    assert(err not_eq nullptr);
    return std::unexpected(unique_error{err});
  }
  /// Load schematic from world edit 1.12 (*.schematic)
  [[nodiscard]] static std::expected<unique_schematic, unique_error>
  load_world_edit12(std::istream& is, const world_edit12_load_option& opt) {
    error* err = nullptr;
    rust_reader isw{is};
    auto ret =
        mc_schem_schematic_load_world_edit12_from_reader(&isw, &opt, &err);
    if (ret) {
      assert(err == nullptr);
      return unique_schematic{ret};
    }
    assert(err not_eq nullptr);
    return std::unexpected(unique_error{err});
  }
  /// Load schematic from file, identify format by filename extension
  /// automatically
  [[nodiscard]] static std::expected<unique_schematic, unique_error>
  load_from_file(const char* filename) {
    error* err = nullptr;
    auto schem = mc_schem_schematic_load_from_file(filename, &err);
    if (schem) {
      assert(err == nullptr);
      return unique_schematic{schem};
    }
    assert(err not_eq nullptr);
    return std::unexpected(unique_error{err});
  }
  /// Save as litematica (*.litematic)
  std::expected<void, unique_error> save_litematica(
      std::ostream& os, const litematica_save_option& opt) const& {
    rust_writer osw{os};
    auto err = mc_schem_schematic_save_litematica_to_writer(this, &osw, &opt);
    if (not err) {
      return {};
    }
    return std::unexpected(unique_error{err});
  }
  /// Save as world edit 1.13+ (*.schem)
  std::expected<void, unique_error> save_world_edit13(
      std::ostream& os, const world_edit13_save_option& opt) const& {
    rust_writer osw{os};
    auto err = mc_schem_schematic_save_world_edit13_to_writer(this, &osw, &opt);
    if (not err) {
      return {};
    }
    return std::unexpected(unique_error{err});
  }
  /// Save to vanilla structure (*.nbt)
  std::expected<void, unique_error> save_vanilla_structure(
      std::ostream& os, const vanilla_structure_save_option& opt) const& {
    rust_writer osw{os};
    auto err =
        mc_schem_schematic_save_vanilla_structure_to_writer(this, &osw, &opt);
    if (not err) {
      return {};
    }
    return std::unexpected(unique_error{err});
  }
  /// Save schematic to file, identify format by filename extension
  /// automatically
  std::expected<void, unique_error> save_to_file(const char* filename) const& {
    auto error = mc_schem_schematic_save_to_file(this, filename);
    if (not error) {
      return {};
    }
    return std::unexpected(unique_error{error});
  }
};

}  // namespace mc_schem

#endif  // MC_SCHEM_MC_SCHEM_HPP
