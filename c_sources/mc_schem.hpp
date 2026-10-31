/*
 mc_schem is a rust library to generate, load, manipulate and save minecraft
 schematic files. Copyright (C) 2026 ToKiNoBug

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
#include <map>
#include <memory>
#include <optional>
#include <ostream>
#include <span>
#include <stdexcept>
#include <string>
#include <string_view>
#include <tuple>
#include <type_traits>
#include <vector>

#include "mc_schem.hpp"

#ifdef MC_SCHEM_HAS_mdspan
#include <mdspan>
#endif

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
// Place holders for raw metadata. TODO: add real APIs for them
class litematica_metadata;
class world_edit12_metadata;
class world_edit13_metadata;
class vanilla_structure_metadata;

/// Copy rust str to std::string
struct alignas(8) rust_string_receiver {
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

 private:
  [[maybe_unused]] uint8_t reserved[48]{};

 public:
  explicit rust_string_receiver(std::string& dest)
      : func_receive_string{callback_receive_string}, custom_data{&dest} {}
};

static_assert(sizeof(rust_string_receiver) == 64);

namespace internal {
inline void save_error_info(std::string_view msg, char* error_message_dest,
                            size_t error_message_capacity) {
  const size_t len = std::min(msg.size(), error_message_capacity);
  std::copy_n(msg.data(), len, error_message_dest);
}
}  // namespace internal

/// Encapsulate std::istream as std::io::Read for rust
struct rust_reader {
  size_t (*func_read)(uint8_t* dest, size_t dest_capacity, bool* ok_nonnull,
                      char* error_message_dest, size_t error_message_capacity,
                      void* custom_data){nullptr};
  void* custom_data{nullptr};

 private:
  [[maybe_unused]] uint8_t reserved[48]{};

 public:
  explicit rust_reader(std::istream& is) : custom_data{&is} {
    func_read = [](uint8_t* dest, size_t dest_capacity, bool* ok,
                   char* error_message_dest, size_t error_message_capacity,
                   void* custom_data) -> size_t {
      auto& is = *reinterpret_cast<std::istream*>(custom_data);

      try {
        // Do not use `readsome` here. On MSVC a freshly opened file has no get
        // area yet, so `readsome` returns 0 while the stream is still good, and
        // the rust side reads a zero byte count as end of input: every stream
        // would look empty. `read` fills the buffer and `gcount` reports how
        // many bytes really arrived.
        is.read(reinterpret_cast<char*>(dest),
                static_cast<std::streamsize>(dest_capacity));
        const std::streamsize read_bytes = is.gcount();
        // A short read is the normal way to reach the end of the input, so a
        // positive count, end of file, or a healthy stream all count as
        // success. Only reading nothing while the stream is neither exhausted
        // nor healthy is a failure.
        *ok = (read_bytes > 0) or is.eof() or is.good();
        return static_cast<size_t>(read_bytes);
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

static_assert(sizeof(rust_reader) == 64);

/// Encapsulate std::ofstream or std::vector into std::io::Write for rust
struct rust_writer {
  size_t (*func_write)(const uint8_t* buf, size_t buf_bytes, bool* ok_nonnull,
                       char* error_message_dest, size_t error_message_capacity,
                       void* custom_data){nullptr};
  bool (*func_flush)(void* custom_data, char* error_message_dest,
                     size_t error_message_capacity){nullptr};
  void* custom_data{nullptr};

 private:
  [[maybe_unused]] uint8_t reserved[40]{};

 public:
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
        dest.reserve(dest.size() + buf_bytes);
        dest.insert(dest.end(), buf, buf + buf_bytes);
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
static_assert(sizeof(rust_writer) == 64);

/// Statistic result for updating mushroom state
struct alignas(8) mirror_mushroom_statistic {
  uint64_t mushroom_total_num{0};
  uint64_t mushroom_corrected_num{0};
  /// More blocks appended to palette
  uint16_t palette_growth{0};

  [[maybe_unused]] uint8_t reserved[46]{};
};
static_assert(sizeof(mirror_mushroom_statistic) == 64);

struct alignas(8) mushroom_state_update_option {
  /// Canonicalize mushroom block id. Palette may contain non-canonical mushroom
  /// block id, for example that skips some field in block state
  bool canonicalize_block_id{false};

  [[maybe_unused]] uint8_t reserved[63]{};
};
static_assert(sizeof(mushroom_state_update_option) == 64);

struct alignas(8) vanilla_structure_load_option {
  common_block background_block{common_block::structure_void};
  bool is_compressed{true};

  [[maybe_unused]] uint8_t reserved[61]{};
};
static_assert(sizeof(vanilla_structure_load_option) == 64);

struct alignas(8) vanilla_structure_save_option {
  uint32_t compress_level{9};
  /// If true, air blocks are explicitly written. Otherwise treated as
  /// structure_void. Vanilla structure is a sparse format that stores
  /// non-void blocks explicitly by position and block index
  bool keep_air{true};
  [[maybe_unused]] uint8_t reserved[59]{};
};
static_assert(sizeof(vanilla_structure_save_option) == 64);

struct alignas(8) litematica_load_option {
  /// True if loading from gzip-compressed input
  bool is_compressed{true};

  [[maybe_unused]] uint8_t reserved[63]{};
};
static_assert(sizeof(litematica_load_option) == 64);

struct alignas(8) litematica_save_option {
  uint32_t compress_level{9};
  bool rename_duplicated_regions{true};

  [[maybe_unused]] uint8_t reserved[59]{};
};
static_assert(sizeof(litematica_save_option) == 64);

struct alignas(8) world_edit13_load_option {
  /// True if loading from gzip-compressed input
  bool is_compressed{true};

  [[maybe_unused]] uint8_t reserved[63]{};
};
static_assert(sizeof(world_edit13_load_option) == 64);

struct alignas(8) world_edit13_save_option {
  uint32_t compress_level{9};
  /// Schematic may contains multiple region but WE stores only 1 region in
  /// dense. Space inside volume but outside any region is filled with that
  /// background block.
  common_block background_block{common_block::air};

  [[maybe_unused]] uint8_t reserved[58]{};
};
static_assert(sizeof(world_edit13_save_option) == 64);

struct alignas(8) world_edit12_load_option {
  /// Data version of this schematic. Data version is not stored in
  /// `.schematic`, so we should assign it.
  int32_t data_version{1343};  // Java_1_12_2
  /// True if loading from gzip-compressed input
  bool is_compressed{true};

  [[maybe_unused]] uint8_t reserved[59]{};
};
static_assert(sizeof(world_edit12_load_option) == 64);

/// Mirror struct to pass information from rust side
struct alignas(8) mirror_invalid_block_info {
  /// Total count of invalid blocks
  uint64_t count{0};
  /// Position of first invalid block
  int32_t first_block_position[3]{0, 0, 0};
  /// Index of first invalid block
  uint16_t first_block_index{0};

  [[maybe_unused]] uint8_t reserved[42]{};
};
static_assert(sizeof(mirror_invalid_block_info) == 64);
// Version and ABI anchors
extern "C" {

/// Returns 0x<major>'<minor>'<patch>'<tweak>. 1.3.0 -> 0x01'03'00'00
[[nodiscard]] uint32_t mc_schem_version();
// version anchor for 1.3.0.
uint32_t mc_schem_version_anchor_v1_3_0() noexcept;
}
namespace internal {
/// Version anchor carriers. This const global variable forces downstream
/// projects to call version anchor functions, so that dynamic link failure
/// happens when expected dll version is lower than exact version. We know
/// anchor functions is pure (no side effects), but compiler and linker don't.
/// So LTO won't remove call to anchor functions.
///
/// Every ABI change should make a new anchor (and preserve
/// all previous anchors). If breaking changes (different major version), remove
/// previous anchors.
inline const uint32_t anchor_v1_3_0 [[maybe_unused]] =
    mc_schem_version_anchor_v1_3_0();
}  // namespace internal

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
void mc_schem_destroy_litematica_metadata(litematica_metadata* mdata);
void mc_schem_destroy_world_edit12_metadata(world_edit12_metadata* mdata);
void mc_schem_destroy_world_edit13_metadata(world_edit13_metadata* mdata);
void mc_schem_destroy_vanilla_structure_metadata(
    vanilla_structure_metadata* mdata);

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
void mc_schem_swap_block(block* a, block* b) noexcept;
void mc_schem_swap_region(region* a, region* b) noexcept;
void mc_schem_swap_entity(entity* a, entity* b) noexcept;
void mc_schem_swap_block_entity(block_entity* a, block_entity* b) noexcept;
void mc_schem_swap_pending_tick(pending_tick* a, pending_tick* b) noexcept;
void mc_schem_swap_metadata_ir(metadata_ir* a, metadata_ir* b) noexcept;
void mc_schem_swap_nbt_hashmap(nbt_hashmap* a, nbt_hashmap* b) noexcept;
void mc_schem_swap_schematic(schematic* a, schematic* b) noexcept;

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
/// Create nbt from sNBT string. Different from other APIs, here str don't have
/// to be end with \0. This aims to support std::string_view and avoid deep
/// copy, because snbt strings may be long
[[nodiscard]] nbt_hashmap* mc_schem_create_nbt_hashmap_from_snbt(
    const char* snbt_in_utf8, size_t len_in_bytes,
    const rust_string_receiver* error_message_receiver);
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
[[nodiscard]] block* mc_schem_create_block_from_id(
    const char* full_id, block_id_parse_error* dest_error_nonnull);
void mc_schem_block_get_id(const block*, const rust_string_receiver* receiver);
void mc_schem_block_get_namespace(const block*,
                                  const rust_string_receiver* receiver);
void mc_schem_block_set_id(block*, const char*);
void mc_schem_block_set_namespace(block*, const char*);
void mc_schem_block_get_full_id(const block* block,
                                const rust_string_receiver* receiver);
bool mc_schem_block_reset(block* block, const char* full_id,
                          block_id_parse_error* detail_nullable);
[[nodiscard]] bool mc_schem_block_get_attribute(
    const block*, const char* key, const rust_string_receiver* value);
void mc_schem_block_visit_attributes(
    const block* block,
    void (*callback)(const char8_t* key, size_t key_bytes, const char8_t* value,
                     size_t value_bytes, void* custom_data),
    void* custom_data);
[[deprecated("Use mc_schem_block_set_attribute instead")]] void
mc_schem_block_erase_attribute(block* block, const char* key);
/// if value is nullptr, erase this attribute
void mc_schem_block_set_attribute(block* block, const char* key,
                                  const char* value);
[[nodiscard]] bool mc_schem_block_is_air(const block*);
[[nodiscard]] bool mc_schem_block_is_structure_void(const block*);
/// Compare blocks by value
[[nodiscard]] bool mc_schem_block_is_equal(const block*, const block*);

// Error
////////////////////////////////////////////////////////////////////////////////
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
void mc_schem_region_update_mushroom_state(
    region*, const mushroom_state_update_option* options,
    mirror_mushroom_statistic* dest);
void mc_schem_region_fill_with(region*, const block* blk);
void mc_schem_region_convert_to_sparse(region*);
void mc_schem_region_convert_to_dense(region*);
/// Returns true if contains invalid blocks. dest_counter should be size of
/// palette_size
[[nodiscard]] bool mc_schem_region_stat_blocks(
    const region*, uint64_t* dest_counter,
    mirror_invalid_block_info* invalid_blocks_info);

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
/// Get data of dense array (if dense). If sparse, returns nullptr. Arrays are
/// in C-order, yzx
[[nodiscard]] const uint16_t* mc_schem_region_get_data(const region*);
[[nodiscard]] uint16_t* mc_schem_region_get_data_mut(region*);

// Entity
////////////////////////////////////////////////////////////////////////////////
/// Create entity from nbt tags (deep copy tags)
[[nodiscard]] entity* mc_schem_create_entity(const nbt_hashmap* nbt,
                                             error** error_dest_non_null);
/// Create entity from nbt tags (nbt must be from box, always consumed in rust
/// side even failed)
[[nodiscard]] entity* mc_schem_create_entity_move(nbt_hashmap* nbt,
                                                  error** error_dest_non_null);

void mc_schem_entity_get_position(const entity*, int32_t* x, int32_t* y,
                                  int32_t* z, double* fp_x, double* fp_y,
                                  double* fp_z);
const nbt_hashmap* mc_schem_entity_get_tags(const entity*);
nbt_hashmap* mc_schem_entity_get_tags_mut(entity*);
/// Deep copy new_value into entity, returns old value by box (transfer
/// ownership
nbt_hashmap* mc_schem_entity_set_tags(entity*, const nbt_hashmap* new_value);
/// Serialize entity to nbt
[[nodiscard]] nbt_hashmap* mc_schem_entity_to_nbt(const entity*);

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
// getter
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
void mc_schem_metadata_ir_get_schem_offset(const metadata_ir*, int32_t* dest_x,
                                           int32_t* dest_y, int32_t* dest_z);
bool mc_schem_metadata_ir_get_schem_we_offset(const metadata_ir*,
                                              int32_t* dest_x, int32_t* dest_y,
                                              int32_t* dest_z);
bool mc_schem_metadata_ir_get_schem_world_edit_version(
    const metadata_ir*, const rust_string_receiver*);
bool mc_schem_metadata_ir_get_schem_editing_platform(
    const metadata_ir*, const rust_string_receiver*);
bool mc_schem_metadata_ir_get_schem_origin(const metadata_ir*, int32_t* dest_x,
                                           int32_t* dest_y, int32_t* dest_z);
void mc_schem_metadata_ir_get_schem_material(const metadata_ir*,
                                             const rust_string_receiver*);
void mc_schem_metadata_ir_get_description(const metadata_ir*,
                                          const rust_string_receiver*);
// setter
[[nodiscard]] error* mc_schem_metadata_ir_set_mc_data_version(metadata_ir*,
                                                              int32_t version);
void mc_schem_metadata_ir_set_time_created(metadata_ir*, int64_t time);
void mc_schem_metadata_ir_set_time_modified(metadata_ir*, int64_t time);
void mc_schem_metadata_ir_set_author(metadata_ir*, const char*);
void mc_schem_metadata_ir_set_name(metadata_ir*, const char*);
void mc_schem_metadata_ir_set_litematica_version(metadata_ir*, int32_t);
// if exist=false, ignore subver; otherwise use subver
void mc_schem_metadata_ir_set_litematica_subversion(metadata_ir*,
                                                    int32_t subver, bool exist);
void mc_schem_metadata_ir_set_schem_version(metadata_ir*, int32_t);
void mc_schem_metadata_ir_set_schem_offset(metadata_ir*, int32_t x, int32_t y,
                                           int32_t z);
void mc_schem_metadata_ir_set_we_offset(metadata_ir*, int32_t x, int32_t y,
                                        int32_t z, bool exist);
// if we_version=null, erase this tag; otherwise set to we_
void mc_schem_metadata_ir_set_schem_world_edit_version(
    metadata_ir*, const char* we_version_nullable);
void mc_schem_metadata_ir_set_schem_editing_platform(
    metadata_ir*, const char* we_editing_platform_nullable);
void mc_schem_metadata_ir_set_schem_origin(metadata_ir*, int32_t x, int32_t y,
                                           int32_t z, bool exist);
void mc_schem_metadata_ir_set_schem_material(metadata_ir*, const char*);
void mc_schem_metadata_ir_set_description(metadata_ir*, const char*);
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
    rust_reader* src, const litematica_load_option*, error** error_dest_nonnull,
    litematica_metadata** raw_metadata_dest_nonnull);
/// Load vanilla structure file (*.nbt)
[[nodiscard]] schematic* mc_schem_schematic_load_vanilla_structure_from_reader(
    rust_reader* src, const vanilla_structure_load_option*,
    error** error_dest_nonnull,
    vanilla_structure_metadata** raw_metadata_dest_nonnull);
/// Load WorldEdit 1.13+ (*.schem)
[[nodiscard]] schematic* mc_schem_schematic_load_world_edit13_from_reader(
    rust_reader* src, const world_edit13_load_option*,
    error** error_dest_nonnull,
    world_edit13_metadata** raw_metadata_dest_nonnull);
/// Load WorldEdit 1.12 (*.schematic)
[[nodiscard]] schematic* mc_schem_schematic_load_world_edit12_from_reader(
    rust_reader* src, const world_edit12_load_option*,
    error** error_dest_nonnull,
    world_edit12_metadata** raw_metadata_dest_nonnull);

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

[[nodiscard]] inline uint32_t version_number() { return mc_schem_version(); }

[[nodiscard]] inline std::string version_string() {
  const uint32_t v = version_number();
  const uint32_t major = (v bitand 0xFF'00'00'00) >> 24;
  const uint32_t minor = (v bitand 0x00'FF'00'00) >> 16;
  const uint32_t patch = (v bitand 0x00'00'FF'00) >> 8;
  const uint32_t tweak = (v bitand 0x00'00'00'FF) >> 0;
  if (tweak > 0) {
    return std::format("{}.{}.{}.{}", major, minor, patch, tweak);
  }
  return std::format("{}.{}.{}", major, minor, patch);
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
  static void operator()(world_edit12_metadata* ptr) {
    mc_schem_destroy_world_edit12_metadata(ptr);
  }
  static void operator()(world_edit13_metadata* ptr) {
    mc_schem_destroy_world_edit13_metadata(ptr);
  }
  static void operator()(litematica_metadata* ptr) {
    mc_schem_destroy_litematica_metadata(ptr);
  }
  static void operator()(vanilla_structure_metadata* ptr) {
    mc_schem_destroy_vanilla_structure_metadata(ptr);
  }
};

using unique_block = std::unique_ptr<block, deleter>;
using unique_entity = std::unique_ptr<entity, deleter>;
using unique_block_entity = std::unique_ptr<block_entity, deleter>;
using unique_pending_tick = std::unique_ptr<pending_tick, deleter>;
using unique_schematic = std::unique_ptr<schematic, deleter>;
using unique_error = std::unique_ptr<error, deleter>;
using shared_error = std::shared_ptr<error>;
using unique_region = std::unique_ptr<region, deleter>;
using unique_metadata_ir = std::unique_ptr<metadata_ir, deleter>;
using unique_nbt_hashmap = std::unique_ptr<nbt_hashmap, deleter>;
using unique_litematica_metadata =
    std::unique_ptr<litematica_metadata, deleter>;
using unique_world_edit12_metadata =
    std::unique_ptr<world_edit12_metadata, deleter>;
using unique_world_edit13_metadata =
    std::unique_ptr<world_edit13_metadata, deleter>;
using unique_vanilla_structure_metadata =
    std::unique_ptr<vanilla_structure_metadata, deleter>;

namespace internal {
template <class E>
  requires std::is_move_constructible_v<E> and
           (std::is_same_v<E, shared_error> or std::is_same_v<E, std::string> or
            std::is_same_v<E, block_id_parse_error>)
[[noreturn]] void auto_throw(E e) {
  if constexpr (std::is_same_v<E, shared_error>) {
    throw std::runtime_error{e->message()};
  }
  if constexpr (std::is_same_v<E, std::string>) {
    throw std::runtime_error{e};
  }
  if constexpr (std::is_same_v<E, block_id_parse_error>) {
    throw std::runtime_error{
        std::format("Failed to parse block id with "
                    "error code {}",
                    static_cast<int>(e))};
  }
  // std::unreachable();
}
}  // namespace internal

/// Unwrap an expected (by right value) and returns value content. If not ok,
/// throw an exception with full error info. This is suggested usage to
/// handle excepted return types. Simply call `std::expected::value()` will
/// discard error info.
template <class T, class E>
  requires std::is_move_constructible_v<T> or std::is_void_v<T>
T unwrap(std::expected<T, E> exp) {
  if (not exp.has_value()) [[unlikely]] {
    ::mc_schem::internal::auto_throw(std::move(exp.error()));
  }

  if constexpr (not std::is_void_v<T>) {
    return std::move(exp.value());
  } else {
    return;
  }
}

// template <class exp_void>
//   requires std::is_same_v<exp_void,
//                           std::expected<void, typename exp_void::error_type>>
// void unwrap(exp_void exp) {
//   if (not exp.has_value()) {
//     ::mc_schem::internal::auto_throw(std::move(exp.error()));
//   }
// }

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
  // void operator delete(void*) = delete;
  // void operator delete[](void*) = delete;

  /// Create an empty nbt hashmap.
  [[nodiscard]] static unique_nbt_hashmap create() {
    return unique_nbt_hashmap{mc_schem_create_nbt_hashmap()};
  }

  /// Parse an uncompressed binary nbt document stored in memory. Returns the
  /// top level tags, or the parser message if the bytes are not a valid nbt
  /// document
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

  /// Parse an uncompressed binary nbt document from a stream. Returns the top
  /// level tags, or the parser message if reading or parsing fails
  [[nodiscard]] static std::expected<unique_nbt_hashmap, std::string>
  create_from_binary(std::istream& is) {
    std::string error_msg;
    rust_string_receiver receiver{error_msg};

    rust_reader isw{is};

    auto ptr = mc_schem_create_nbt_hashmap_from_binary_stream(&isw, &receiver);
    if (ptr == nullptr) {
      return std::unexpected(std::move(error_msg));
    }
    return unique_nbt_hashmap{ptr};
  }

  /// Parse an uncompressed binary nbt document from a stream. Returns the top
  /// level tags, or the parser message if reading or parsing fails
  [[deprecated(
      "Use create_from_binary(std::istream) "
      "instead")]] [[nodiscard]] static std::expected<unique_nbt_hashmap,
                                                      std::string>
  create_from_binary_stream(std::istream& is) {
    return create_from_binary(is);
  }

  void swap(nbt_hashmap& another) & noexcept {
    mc_schem_swap_nbt_hashmap(this, &another);
  }

  /// Parse string nbt. Returns the top level tags, or the parser message if
  /// reading or parsing fails. To avoid deep copy, snbt don't have to end with
  /// \0
  [[nodiscard]] static std::expected<unique_nbt_hashmap, std::string>
  create_from_snbt(std::string_view snbt_utf8) {
    std::string error_msg;
    rust_string_receiver receiver{error_msg};
    auto ptr = mc_schem_create_nbt_hashmap_from_snbt(
        snbt_utf8.data(), snbt_utf8.size(), &receiver);
    if (ptr == nullptr) {
      return std::unexpected(std::move(error_msg));
    }
    return unique_nbt_hashmap{ptr};
  }

  /// Count of top level tags in this hashmap
  [[nodiscard]] size_t size() const& {
    return mc_schem_nbt_hashmap_get_size(this);
  }

  /// Write this hashmap as uncompressed binary nbt to `os`. Returns the
  /// serializer message on failure
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

  /// Write this hashmap as uncompressed binary nbt to `dest`, replacing its
  /// previous contents. Throws std::runtime_error if serialization fails
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

  /// Serialize this hashmap to uncompressed binary nbt
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

  /// Create an air block (minecraft:air)
  [[nodiscard]] static unique_block create() {
    return unique_block{mc_schem_create_block()};
  }

  /// Create a block by parsing a full block id, for example
  /// `minecraft:oak_stairs[facing=north,half=bottom]`. Both the namespace and
  /// the attribute list may be omitted. Returns the parse error if `id` is not
  /// a valid block id
  [[nodiscard]] static std::expected<unique_block, block_id_parse_error> create(
      const std::string& id) {
    block_id_parse_error err{};
    auto ptr = mc_schem_create_block_from_id(id.c_str(), &err);
    if (ptr) {
      return unique_block{ptr};
    }
    return std::unexpected(err);
  }

  /// Create one of the common blocks, like air or structure void
  [[nodiscard]] static unique_block create(common_block cb) {
    return unique_block{mc_schem_create_block_from_common(cb)};
  }
  [[deprecated(
      "Use block::create(common_block) "
      "instead")]] [[nodiscard]] static unique_block
  from_common(common_block cb) {
    return create(cb);
  }

  [[nodiscard]] auto clone() const& {
    return unique_block{mc_schem_clone_block(this)};
  }

  void swap(block& another) & noexcept { mc_schem_swap_block(this, &another); }

  /// Id of this block without the namespace, for example `stone`
  [[nodiscard]] std::string id() const& {
    std::string ret;
    rust_string_receiver receiver{ret};
    mc_schem_block_get_id(this, &receiver);
    return ret;
  }

  /// Namespace of this block. Vanilla blocks use `minecraft`
  [[nodiscard]] std::string namespace_() const& {
    std::string ret;
    rust_string_receiver receiver{ret};
    mc_schem_block_get_namespace(this, &receiver);
    return ret;
  }

  /// Replace the id of this block. Namespace and attributes are kept
  void set_id(const std::string& id) & {
    mc_schem_block_set_id(this, id.c_str());
  }
  /// Replace the namespace of this block. Id and attributes are kept
  void set_namespace(const std::string& ns) & {
    mc_schem_block_set_namespace(this, ns.c_str());
  }

  /// Full id of this block, like `minecraft:oak_stairs[facing=north]`. The
  /// namespace is omitted when empty and so is the attribute list
  [[nodiscard]] std::string full_id() const& {
    std::string ret;
    rust_string_receiver receiver{ret};
    mc_schem_block_get_full_id(this, &receiver);
    return ret;
  }

  /// Replace this block with the one parsed from `full_id`. Returns the parse
  /// error and leaves this block untouched if `full_id` is invalid
  [[nodiscard]] std::expected<void, block_id_parse_error> reset(
      const std::string& full_id) & {
    block_id_parse_error err{};
    if (mc_schem_block_reset(this, full_id.c_str(), &err)) {
      return {};
    }
    return std::unexpected{err};
  }

  /// Visit every attribute of this block. The visitor receives the attribute
  /// name and value by value, in ascending name order
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

  /// Value of attribute `key`, or nullopt if this block has no such attribute
  [[nodiscard]] std::optional<std::string> get_attribute(
      const std::string& key) const& {
    std::string ret;
    rust_string_receiver receiver{ret};
    const bool exist =
        mc_schem_block_get_attribute(this, key.c_str(), &receiver);
    if (exist) return ret;
    return std::nullopt;
  }

  /// Set attribute `key` to `value`, overwriting the previous value if any
  void set_attribute(const std::string& key, const std::string& value) & {
    mc_schem_block_set_attribute(this, key.c_str(), value.c_str());
  }
  /// Remove attribute `key` if it exists
  void erase_attribute(const std::string& key) & {
    mc_schem_block_set_attribute(this, key.c_str(), nullptr);
  }
  /// Returns a deep copy of attributes.
  [[nodiscard]] std::map<std::string, std::string> clone_attributes() const& {
    std::map<std::string, std::string> ret;
    this->visit_attributes([&](std::string key, std::string value) {
      ret.emplace(std::move(key), std::move(value));
    });
    return ret;
  }

  /// True if this is air or cave air, without any attribute
  [[nodiscard]] bool is_air() const& { return mc_schem_block_is_air(this); }
  /// True if this is minecraft:structure_void without any attribute
  [[nodiscard]] bool is_structure_void() const& {
    return mc_schem_block_is_structure_void(this);
  }

  /// Compare blocks by value: namespace, id and all attributes must be equal
  [[nodiscard]] bool operator==(const block& another) const& {
    return mc_schem_block_is_equal(this, &another);
  }
  /// Negation of operator==
  [[nodiscard]] bool operator!=(const block& another) const& {
    return not operator==(another);
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

  /// Human readable description of this error
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

  /// Create a dense region of the given shape (xyz). The region is filled with
  /// air and its palette holds only air
  [[nodiscard]] static unique_region create(int32_t shape_x, int32_t shape_y,
                                            int32_t shape_z) {
    return unique_region{mc_schem_create_region(shape_x, shape_y, shape_z)};
  }

  /// Create a dense region of the given `shape` (xyz) whose palette is a deep
  /// copy of `palette`. Every block of the region is set to `palette[0]`, so
  /// the palette must not be empty. Returns an error if it is
  [[nodiscard]] static std::expected<unique_region, shared_error>
  create_with_palette(std::span<const int32_t, 3> shape,
                      const std::span<const block* const> palette) {
    const auto x = shape[0];
    const auto y = shape[1];
    const auto z = shape[2];

    error* err{nullptr};
    auto result = mc_schem_create_region_with_palette(x, y, z, palette.data(),
                                                      palette.size(), &err);
    if (result) {
      assert(err == nullptr);
      return unique_region{result};
    }
    assert(err);
    return std::unexpected{std::shared_ptr{unique_error{err}}};
  }
  /// Convenience overload taking the shape as std::array
  [[nodiscard]] static std::expected<unique_region, shared_error>
  create_with_palette(const std::array<int32_t, 3>& shape,
                      const std::span<const block* const> palette) {
    return create_with_palette(std::span{shape}, palette);
  }

  [[nodiscard]] auto clone() const& {
    return unique_region{mc_schem_clone_region(this)};
  }

  void swap(region& another) & noexcept {
    mc_schem_swap_region(this, &another);
  }

  /// Name of this region. Only litematica persists it
  [[nodiscard]] std::string name() const& {
    std::string ret;
    rust_string_receiver receiver{ret};
    mc_schem_region_get_name(this, &receiver);
    return ret;
  }

  /// Offset of this region inside the schematic, in blocks (xyz)
  template <typename int_t = int32_t>
    requires std::is_integral_v<int_t>
  [[nodiscard]] std::array<int_t, 3> offset() const& {
    int32_t x{0}, y{0}, z{0};
    mc_schem_region_get_offset(this, &x, &y, &z);
    return {static_cast<int_t>(x), static_cast<int_t>(y),
            static_cast<int_t>(z)};
  }

  /// Set the name of this region. Only litematica persists it
  void set_name(const std::string& name) & {
    mc_schem_region_set_name(this, name.c_str());
  }

  /// Set the offset of this region inside the schematic, in blocks (xyz)
  void set_offset(std::span<const int32_t, 3> offset) & {
    mc_schem_region_set_offset(this, offset[0], offset[1], offset[2]);
  }
  void set_offset(const std::array<int32_t, 3>& offset) & {
    set_offset(std::span{offset});
  }

  /// Size of this region in x, y, z order
  template <typename int_t = int32_t>
    requires std::is_integral_v<int_t>
  [[nodiscard]] std::array<int_t, 3> size_xyz() const& {
    int32_t x{0}, y{0}, z{0};
    mc_schem_region_get_size(this, &x, &y, &z);
    assert(x >= 0);
    assert(y >= 0);
    assert(z >= 0);
    return {static_cast<int_t>(x), static_cast<int_t>(y),
            static_cast<int_t>(z)};
  }

  /// Size of this region in y, z, x order. This is the axis order of the
  /// internal dense array and of all schematic file formats
  template <typename int_t = int32_t>
    requires std::is_integral_v<int_t>
  [[nodiscard]] std::array<int_t, 3> size_yzx() const& {
    const auto [x, y, z] = size_xyz();
    return {static_cast<int_t>(y), static_cast<int_t>(z),
            static_cast<int_t>(x)};
  }

  /// True if blocks are stored in a dense array instead of a sparse map
  [[nodiscard]] bool is_dense() const& {
    return mc_schem_region_is_dense(this);
  }
  /// True if blocks are stored in a sparse map
  [[nodiscard]] bool is_sparse() const& { return not is_dense(); }

  /// Resize this region to `shape` (xyz). All blocks become the first palette
  /// entry; palette, entities and block entities are kept
  void reshape(std::span<const int32_t, 3> shape) & {
    mc_schem_region_reshape(this, shape[0], shape[1], shape[2]);
  }
  void reshape(const std::array<int32_t, 3>& shape) & {
    reshape(std::span{shape});
  }

  /// Number of distinct blocks in the palette
  [[nodiscard]] size_t palette_size() const& {
    return mc_schem_region_palette_get_size(this);
  }

  /// Block at `index` of the palette, or null if `index` is out of range
  [[nodiscard]] const block* palette(size_t index) const& {
    return mc_schem_region_palette_get_block(this, index);
  }

  /// Index of `blk` in the palette. If no equal block exists yet, `blk` is
  /// deep copied and appended to the palette first
  [[nodiscard]] uint16_t find_or_append_to_palette(const block& blk) & {
    return mc_schem_region_find_or_append_to_palette(this, &blk);
  }

  /// Index of `blk` in the palette, or nullopt if the palette contains no
  /// equal block
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

  /// Pointers to all blocks of the palette, in palette order
  [[nodiscard]] std::vector<const block*> full_palette() const {
    std::vector<const block*> ret;
    const size_t n = palette_size();
    ret.reserve(n);
    for (size_t i = 0; i < n; i++) {
      ret.emplace_back(palette(i));
    }
    return ret;
  }

  /// Number of entities in this region
  [[nodiscard]] size_t entities_count() const& {
    return mc_schem_region_get_entities_count(this);
  }
  /// Entity at `index`, or null if `index` is out of range
  [[nodiscard]] const entity* get_entity(size_t index) const& {
    return mc_schem_region_get_entity(this, index);
  }
  /// Mutable entity at `index`, or null if `index` is out of range
  [[nodiscard]] entity* get_entity(size_t index) & {
    return mc_schem_region_get_entity_mut(this, index);
  }
  /// Remove and return the entity at `index`. `index` must be less than
  /// entities_count()
  unique_entity erase_entity(size_t index) & {
    auto ptr = mc_schem_region_erase_entity(this, index);
    return unique_entity{ptr};
  }
  /// Append a deep copy of `entity` to this region and return its index
  size_t add_entity(const entity& entity) & {
    return mc_schem_region_add_entity(this, &entity);
  }
  /// Move `entity` into this region and return its index
  size_t add_entity(unique_entity entity) & {
    return mc_schem_region_add_entity_move(this, entity.release());
  }

  /// Number of block entities in this region
  [[nodiscard]] size_t block_entities_count() const& {
    return mc_schem_region_get_block_entities_count(this);
  }
  /// Visit every block entity of this region together with its position (xyz)
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
  /// Block entity at `pos` (xyz), or null if there is none
  [[nodiscard]] const block_entity* get_block_entity(
      std::span<const int32_t, 3> pos) const& {
    return mc_schem_region_get_block_entity(this, pos[0], pos[1], pos[2]);
  }
  [[nodiscard]] const block_entity* get_block_entity(
      const std::array<int32_t, 3>& pos) const& {
    return get_block_entity(std::span{pos});
  }

  /// Mutable block entity at `pos` (xyz), or null if there is none
  [[nodiscard]] block_entity* get_block_entity(
      std::span<const int32_t, 3> pos) & {
    return mc_schem_region_get_block_entity_mut(this, pos[0], pos[1], pos[2]);
  }
  [[nodiscard]] block_entity* get_block_entity(
      const std::array<int32_t, 3>& pos) & {
    return get_block_entity(std::span{pos});
  }

  /// Insert a deep copy of `e` at `pos` (xyz), returning the block entity that
  /// was stored there before, if any
  unique_block_entity add_block_entity(std::span<const int32_t, 3> pos,
                                       const block_entity& e) & {
    auto ret =
        mc_schem_region_add_block_entity(this, pos[0], pos[1], pos[2], &e);
    return unique_block_entity{ret};
  }
  unique_block_entity add_block_entity(const std::array<int32_t, 3>& pos,
                                       const block_entity& e) & {
    return add_block_entity(std::span{pos}, e);
  }
  /// Move `new_be` to `pos` (xyz), returning the block entity that was stored
  /// there before, if any
  unique_block_entity add_block_entity(std::span<const int32_t, 3> pos,
                                       unique_block_entity new_be) & {
    auto ret = mc_schem_region_add_block_entity_move(this, pos[0], pos[1],
                                                     pos[2], new_be.release());
    return unique_block_entity{ret};
  }
  unique_block_entity add_block_entity(const std::array<int32_t, 3>& pos,
                                       unique_block_entity new_be) & {
    return add_block_entity(std::span{pos}, std::move(new_be));
  }
  /// Remove the block entity at `pos` (xyz), returning it if there was one
  unique_block_entity erase_block_entity(std::span<const int32_t, 3> pos) & {
    auto ret =
        mc_schem_region_add_block_entity(this, pos[0], pos[1], pos[2], nullptr);
    return unique_block_entity{ret};
  }
  unique_block_entity erase_block_entity(const std::array<int32_t, 3>& pos) & {
    return erase_block_entity(std::span{pos});
  }

  /// Number of pending ticks at `pos` (xyz)
  [[nodiscard]] size_t pending_ticks_count_at(
      std::span<const int32_t, 3> pos) const& {
    return mc_schem_region_get_pending_ticks_count(this, pos[0], pos[1],
                                                   pos[2]);
  }
  [[nodiscard]] size_t pending_ticks_count_at(
      const std::array<int32_t, 3>& pos) const& {
    return pending_ticks_count_at(std::span{pos});
  }

  /// All pending ticks at `pos` (xyz)
  [[nodiscard]] std::vector<std::reference_wrapper<const pending_tick>>
  pending_ticks_at(std::span<const int32_t, 3> pos) const& {
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
  [[nodiscard]] std::vector<std::reference_wrapper<const pending_tick>>
  pending_ticks_at(const std::array<int32_t, 3>& pos) const& {
    return pending_ticks_at(std::span{pos});
  }
  /// Mutable references to all pending ticks at `pos` (xyz)
  [[nodiscard]] std::vector<std::reference_wrapper<pending_tick>>
  pending_ticks_at(std::span<const int32_t, 3> pos) & {
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
  [[nodiscard]] std::vector<std::reference_wrapper<pending_tick>>
  pending_ticks_at(const std::array<int32_t, 3>& pos) & {
    return pending_ticks_at(std::span{pos});
  }

  /// Palette index of the block at `pos` (xyz), or nullopt if `pos` is outside
  /// this region
  [[nodiscard]] std::optional<uint16_t> block_index_at(
      std::span<const int32_t, 3> pos) const& {
    bool ok = false;
    const uint16_t ret =
        mc_schem_region_get_block_index(this, pos[0], pos[1], pos[2], &ok);
    if (not ok) {
      return std::nullopt;
    }
    assert(ret < this->palette_size());
    return ret;
  }
  [[nodiscard]] std::optional<uint16_t> block_index_at(
      const std::array<int32_t, 3>& pos) const& {
    return block_index_at(std::span{pos});
  }

  /// True if `pos` (xyz) is inside this region
  [[nodiscard]] bool contains_coordinate(
      std::span<const int32_t, 3> pos) const& {
    return this->block_index_at(pos).has_value();
  }
  [[nodiscard]] bool contains_coordinate(
      const std::array<int32_t, 3>& pos) const& {
    return contains_coordinate(std::span{pos});
  }

  /// Palette index and block at `pos` (xyz), or nullopt if `pos` is outside
  /// this region
  [[nodiscard]] std::optional<
      std::tuple<uint16_t, std::reference_wrapper<const block>>>
  block_at(std::span<const int32_t, 3> pos) const& {
    const auto idx_opt = this->block_index_at(pos);
    if (not idx_opt) {
      return std::nullopt;
    }
    const uint16_t idx = idx_opt.value();
    auto blkp = this->palette(idx);
    assert(blkp);
    return std::make_tuple(idx, std::ref(*blkp));
  }
  [[nodiscard]] std::optional<
      std::tuple<uint16_t, std::reference_wrapper<const block>>>
  block_at(const std::array<int32_t, 3>& pos) const& {
    return block_at(std::span{pos});
  }

  /// Everything stored at `pos` (xyz): block index, block, block entity (may
  /// be null) and pending ticks. Returns nullopt if `pos` is outside this
  /// region
  [[nodiscard]] std::optional<std::tuple<
      uint16_t, std::reference_wrapper<const block>, const block_entity*,
      std::vector<std::reference_wrapper<const pending_tick>>>>
  block_info_at(std::span<const int32_t, 3> pos) const& {
    const auto blk_opt = this->block_at(pos);
    if (not blk_opt) {
      return std::nullopt;
    }
    const auto [idx, blk] = blk_opt.value();

    const auto bep = this->get_block_entity(pos);
    auto pts = this->pending_ticks_at(pos);
    return std::make_tuple(idx, blk, bep, std::move(pts));
  }
  [[nodiscard]] std::optional<std::tuple<
      uint16_t, std::reference_wrapper<const block>, const block_entity*,
      std::vector<std::reference_wrapper<const pending_tick>>>>
  block_info_at(const std::array<int32_t, 3>& pos) const& {
    return block_info_at(std::span{pos});
  }

  /// Set the block at `pos` (xyz) to palette index `blkid`. Throws
  /// std::runtime_error if `pos` is outside this region or `blkid` is not a
  /// valid palette index
  void set_block(std::span<const int32_t, 3> pos, uint16_t blkid) & {
    if (not mc_schem_region_set_block_by_index(this, pos[0], pos[1], pos[2],
                                               blkid)) {
      // This error is rare. Usually don't process with exception
      throw std::runtime_error{
          std::format("Unable to set block ({}, {}, {}) to index {}. Block out "
                      "of range or block id out of range",
                      pos[0], pos[1], pos[2], blkid)};
    }
  }
  void set_block(const std::array<int32_t, 3>& pos, uint16_t blkid) & {
    set_block(std::span{pos}, blkid);
  }

  /// Set the block at `pos` (xyz), appending `blk` to the palette if no equal
  /// block is there yet. Throws std::runtime_error if `pos` is outside this
  /// region or the palette is already full
  void set_block(std::span<const int32_t, 3> pos, const block& blk) & {
    if (not mc_schem_region_set_block_by_block(this, pos[0], pos[1], pos[2],
                                               &blk)) {
      // This error is rare. Usually don't process with exception
      throw std::runtime_error{
          std::format("Unable to set block ({}, {}, {}). Block out of range or "
                      "palette size out of range",
                      pos[0], pos[1], pos[2])};
    }
  }
  void set_block(const std::array<int32_t, 3>& pos, const block& blk) & {
    set_block(std::span{pos}, blk);
  }

  /// Drop palette entries that no block refers to and renumber the remaining
  /// ones. Returns an error if a block refers to an index that is not in the
  /// palette
  std::expected<void, shared_error> shrink_palette() & {
    auto err = unique_error{mc_schem_region_shrink_palette(this)};
    if (err) {
      return std::unexpected{std::move(err)};
    }
    return {};
  }

  struct mushroom_statistic {
    uint64_t mushroom_total_num{0};
    uint64_t mushroom_corrected_num{0};
    /// More blocks appended to palette
    uint16_t palette_growth{0};
  };
  /// Update block-state of mushroom blocks (full block, not small mushroom). In
  /// real game, contacting mushroom blocks have stoma texture on contacting
  /// face. This should be done by updating block state. This function updates
  /// block index and palette for every mushroom block (if need change), just
  /// like Minecraft does.
  mushroom_statistic update_mushroom_state(
      const mushroom_state_update_option& option) & {
    mirror_mushroom_statistic mirror{};
    mc_schem_region_update_mushroom_state(this, &option, &mirror);
    mushroom_statistic ret{
        .mushroom_total_num = mirror.mushroom_total_num,
        .mushroom_corrected_num = mirror.mushroom_corrected_num,
        .palette_growth = mirror.palette_growth,
    };
    return ret;
  }

  /// Set every block of this region to `blk`
  void fill_with(const block& blk) & { mc_schem_region_fill_with(this, &blk); }
  /// Store blocks as a sparse map of non zero palette indices. No-op if this
  /// region is already sparse
  void convert_to_sparse() & { mc_schem_region_convert_to_sparse(this); }
  /// Store blocks as a dense array. No-op if this region is already dense
  void convert_to_dense() & { mc_schem_region_convert_to_dense(this); }

  /// Visit every block of this region together with its position, block index,
  /// block and block entity (may be null). If `explicit_only` is true, blocks
  /// that are not explicitly stored, that is palette index 0 of a sparse
  /// region, are skipped
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

  struct invalid_block_info {
    std::array<int32_t, 3> first_block_position;
    uint16_t first_block_index;
    uint64_t count;
  };
  /// Statistic blocks in region. Returns block count for every kind of block
  /// in palette. If found any invalid block (index out of range), returns
  /// their count and first block info. Invalid blocks are not included in
  /// block count. Logically there shouldn't be invalid blocks, but for
  /// robustness we don't require region to be good, we just return error
  /// info like in rust.
  [[nodiscard]] std::pair<std::vector<uint64_t>,
                          std::optional<invalid_block_info>>
  stat_blocks() const& {
    std::vector<uint64_t> counter;
    counter.resize(this->palette_size(), 0);
    mirror_invalid_block_info mirror_info{};
    const bool has_invalid =
        mc_schem_region_stat_blocks(this, counter.data(), &mirror_info);

    std::optional<invalid_block_info> info{std::nullopt};
    if (has_invalid) {
      // logically this should be positive. But not very necessary in release
      assert(mirror_info.count > 0);
      info = invalid_block_info{
          .first_block_position = {mirror_info.first_block_position[0],
                                   mirror_info.first_block_position[1],
                                   mirror_info.first_block_position[2]},
          .first_block_index = mirror_info.first_block_index,
          .count = mirror_info.count,
      };
    }
    return std::make_pair(std::move(counter), info);
  }

  /// Returns the count of blocks in region. Air will be counted if
  /// `include_air` is true, structure void is never counted.
  [[nodiscard]] uint64_t total_blocks(bool include_air) const& {
    return mc_schem_region_total_blocks(this, include_air);
  }
#ifdef MC_SCHEM_HAS_mdspan
  /// Try to give a dense view. Only valid if region is dense
  [[nodiscard]] std::optional<
      std::mdspan<const uint16_t, std::dextents<size_t, 3>, std::layout_right>>
  view_as_dense_yzx() const& {
    if (this->is_sparse()) {
      return std::nullopt;
    }
    auto ptr = mc_schem_region_get_data(this);
    const auto [y, z, x] = this->size_yzx();
    if (y * z * x > 0) {
      assert(ptr);
    }
    return std::mdspan<const uint16_t, std::dextents<size_t, 3>,
                       std::layout_right>{ptr, y, z, x};
  }
  /// Try to give a dense view. Only valid if region is dense
  [[nodiscard]] std::optional<
      std::mdspan<uint16_t, std::dextents<size_t, 3>, std::layout_right>>
  view_as_dense_yzx() & {
    if (this->is_sparse()) {
      return std::nullopt;
    }
    auto ptr = mc_schem_region_get_data_mut(this);
    const auto [y, z, x] = this->size_yzx();
    if (y * z * x > 0) {
      assert(ptr);
    }
    return std::mdspan<uint16_t, std::dextents<size_t, 3>, std::layout_right>{
        ptr, y, z, x};
  }
#endif
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

  /// Create an entity from the nbt tags `src`, which are deep copied. Returns
  /// an error if `src` has no usable `Pos` field
  [[nodiscard]] static std::expected<unique_entity, shared_error> create(
      const nbt_hashmap& src) {
    error* err{nullptr};
    auto ptr = mc_schem_create_entity(&src, &err);
    if (ptr) {
      assert(err == nullptr);
      return unique_entity{ptr};
    }
    assert(ptr == nullptr);
    return std::unexpected{std::shared_ptr{unique_error{err}}};
  }

  /// Create an entity from the nbt tags `nbt` it takes over. Returns an error
  /// if `nbt` has no usable `Pos` field
  [[nodiscard]] static std::expected<unique_entity, shared_error> create(
      unique_nbt_hashmap nbt) {
    error* err{nullptr};
    auto ptr = mc_schem_create_entity_move(nbt.release(), &err);
    if (ptr) {
      assert(err == nullptr);
      return unique_entity{ptr};
    }
    assert(ptr == nullptr);
    return std::unexpected{std::shared_ptr{unique_error{err}}};
  }

  [[nodiscard]] auto clone() const& {
    return unique_entity{mc_schem_clone_entity(this)};
  }

  void swap(entity& another) & noexcept {
    mc_schem_swap_entity(this, &another);
  }

  /// Position of this entity (xyz): the block it is in and its exact position
  /// as double
  [[nodiscard]] std::pair<std::array<int32_t, 3>, std::array<double, 3>>
  position() const& {
    std::array<int32_t, 3> block_pos{0, 0, 0};
    std::array<double, 3> pos{0, 0, 0};
    mc_schem_entity_get_position(this, &block_pos[0], &block_pos[1],
                                 &block_pos[2], &pos[0], &pos[1], &pos[2]);
    return std::make_pair(block_pos, pos);
  }

  /// Nbt tags of this entity that are not parsed into other fields
  [[nodiscard]] const nbt_hashmap* tags() const& {
    return mc_schem_entity_get_tags(this);
  }
  /// Mutable nbt tags of this entity. `Pos` and `id` are regenerated when the
  /// entity is serialized
  [[nodiscard]] nbt_hashmap* tags() & {
    return mc_schem_entity_get_tags_mut(this);
  }
  /// Deep copy `src` into this entity and return the tags that were stored
  /// before. `src` must not be the tags of this entity itself
  unique_nbt_hashmap set_tags(const nbt_hashmap& src) & {
    auto old_value = mc_schem_entity_set_tags(this, &src);
    return unique_nbt_hashmap{old_value};
  }
  /// Move `new_value` into this entity and return the tags that were stored
  /// before. Expected usage: set_tags(std::move(new-value-in-unique))
  unique_nbt_hashmap set_tags(unique_nbt_hashmap new_value) & {
    assert(new_value);
    this->tags()->swap(*new_value);
    return new_value;
  }
  /// Serialize entity to nbt. This differs from tags() because: tags() contains
  /// only non-parsed tags, but to_nbt() gives full tags
  [[nodiscard]] unique_nbt_hashmap to_nbt() const& {
    return unique_nbt_hashmap{mc_schem_entity_to_nbt(this)};
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

  /// Create a block entity without any nbt tag
  [[nodiscard]] static unique_block_entity create() {
    return unique_block_entity{mc_schem_create_block_entity()};
  }

  [[nodiscard]] unique_block_entity clone() const& {
    return unique_block_entity{mc_schem_clone_block_entity(this)};
  }

  void swap(block_entity& another) & noexcept {
    mc_schem_swap_block_entity(this, &another);
  }

  /// Nbt tags of this block entity
  [[nodiscard]] const nbt_hashmap* tags() const& {
    return mc_schem_block_entity_get_tags(this);
  }
  /// Mutable nbt tags of this block entity
  [[nodiscard]] nbt_hashmap* tags() & {
    return mc_schem_block_entity_get_tags_mut(this);
  }
  /// Deep copy `new_value` into this block entity and return the tags that
  /// were stored before. `new_value` must not be the tags of this block entity
  /// itself
  unique_nbt_hashmap set_tags(const nbt_hashmap& new_value) & {
    auto old_value = mc_schem_block_entity_set_tags(this, &new_value);
    return unique_nbt_hashmap{old_value};
  }
  /// Move `new_value` into this block entity and return the tags that were
  /// stored before. Expected usage: set_tags(std::move(new-value-in-unique))
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

  /// Create default metadata for the given minecraft data version. The version
  /// is currently not validated
  [[nodiscard]] static std::expected<unique_metadata_ir, shared_error> create(
      int32_t data_version) {
    error* dest_err = nullptr;
    if (auto result = mc_schem_create_metadata_ir(data_version, &dest_err)) {
      assert(dest_err == nullptr);
      return unique_metadata_ir{result};
    }
    assert(dest_err);
    return std::unexpected{std::shared_ptr{unique_error{dest_err}}};
  }

  [[nodiscard]] auto clone() const& {
    return unique_metadata_ir{mc_schem_clone_metadata_ir(this)};
  }

  void swap(metadata_ir& another) & noexcept {
    mc_schem_swap_metadata_ir(this, &another);
  }

  /// Minecraft data version stored in this metadata
  [[nodiscard]] int32_t mc_data_version() const& {
    return mc_schem_metadata_ir_get_mc_data_version(this);
  }
  /// Set the minecraft data version. The version is currently not validated
  void set_mc_data_version(int32_t data_version) & {
    unique_error error{
        mc_schem_metadata_ir_set_mc_data_version(this, data_version)};
    if (error not_eq nullptr) {
      throw std::runtime_error{
          std::format("Failed to set mc data version: {}", error->message())};
    }
  }

  /// Creation time of the schematic, as a unix timestamp in milliseconds
  [[nodiscard]] int64_t time_created() const& {
    return mc_schem_metadata_ir_get_time_created(this);
  }
  /// Set the creation time, as a unix timestamp in milliseconds
  void set_time_created(int64_t time_created) & {
    mc_schem_metadata_ir_set_time_created(this, time_created);
  }

  /// Last modification time of the schematic, as a unix timestamp in
  /// milliseconds
  [[nodiscard]] int64_t time_modified() const& {
    return mc_schem_metadata_ir_get_time_modified(this);
  }
  /// Set the last modification time, as a unix timestamp in milliseconds
  void set_time_modified(int64_t time) & {
    mc_schem_metadata_ir_set_time_modified(this, time);
  }

  /// Author recorded in this metadata
  [[nodiscard]] std::string author() const& {
    std::string ret;
    rust_string_receiver rsr{ret};
    mc_schem_metadata_ir_get_author(this, &rsr);
    return ret;
  }
  /// Set the author recorded in this metadata
  void set_author(const std::string& author) & {
    mc_schem_metadata_ir_set_author(this, author.c_str());
  }

  /// Name of the schematic recorded in this metadata
  [[nodiscard]] std::string name() const& {
    std::string ret;
    rust_string_receiver rsr{ret};
    mc_schem_metadata_ir_get_name(this, &rsr);
    return ret;
  }
  /// Set the name of the schematic recorded in this metadata
  void set_name(const std::string& name) & {
    mc_schem_metadata_ir_set_name(this, name.c_str());
  }

  /// Free form description of the schematic
  [[nodiscard]] std::string description() const& {
    std::string ret;
    rust_string_receiver rsr{ret};
    mc_schem_metadata_ir_get_description(this, &rsr);
    return ret;
  }
  /// Set the free form description of the schematic
  void set_description(const std::string& description) & {
    mc_schem_metadata_ir_set_description(this, description.c_str());
  }

  /// Litematica file version stored in this metadata
  [[nodiscard]] int32_t litematica_version() const& {
    return mc_schem_metadata_ir_get_litematica_version(this);
  }
  /// Set the litematica file version
  void set_litematica_version(int32_t litematica_version) & {
    mc_schem_metadata_ir_set_litematica_version(this, litematica_version);
  }

  /// Litematica sub version, or nullopt if this metadata has none
  [[nodiscard]] std::optional<int32_t> litematica_subversion() const& {
    bool exist{false};
    const auto ret =
        mc_schem_metadata_ir_get_litematica_subversion(this, &exist);
    if (exist) return ret;
    return std::nullopt;
  }
  /// Set the litematica sub version
  void set_litematica_subversion(int32_t v) & {
    mc_schem_metadata_ir_set_litematica_subversion(this, v, true);
  }
  /// Remove the litematica sub version
  void set_litematica_subversion() & {
    mc_schem_metadata_ir_set_litematica_subversion(this, 0, false);
  }

  /// WorldEdit `.schem` file version stored in this metadata
  [[nodiscard]] int32_t schem_version() const& {
    return mc_schem_metadata_ir_get_schem_version(this);
  }
  /// Set the WorldEdit `.schem` file version
  void set_schem_version(int32_t schem_version) & {
    mc_schem_metadata_ir_set_schem_version(this, schem_version);
  }

  /// Offset of the region in a WorldEdit `.schem` file (xyz)
  template <typename int_t = int32_t>
    requires std::is_integral_v<int_t>
  [[nodiscard]] std::array<int_t, 3> schem_offset() const& {
    int32_t x{0}, y{0}, z{0};
    mc_schem_metadata_ir_get_schem_offset(this, &x, &y, &z);
    return {static_cast<int_t>(x), static_cast<int_t>(y),
            static_cast<int_t>(z)};
  }
  /// Set the offset of the region in a WorldEdit `.schem` file (xyz)
  void set_schem_offset(std::span<const int32_t, 3> offset) & {
    mc_schem_metadata_ir_set_schem_offset(this, offset[0], offset[1],
                                          offset[2]);
  }
  void set_schem_offset(const std::array<int32_t, 3>& offset) & {
    set_schem_offset(std::span{offset});
  }

  /// WorldEdit offset, the origin of the schematic in the world (xyz), or
  /// nullopt if this metadata has none
  template <typename int_t = int32_t>
    requires std::is_integral_v<int_t>
  [[nodiscard]] std::optional<std::array<int_t, 3>> schem_we_offset() const& {
    int32_t x{0}, y{0}, z{0};
    const bool exist =
        mc_schem_metadata_ir_get_schem_we_offset(this, &x, &y, &z);
    if (exist) {
      return std::array<int_t, 3>{static_cast<int_t>(x), static_cast<int_t>(y),
                                  static_cast<int_t>(z)};
    }
    return std::nullopt;
  }
  /// Set the WorldEdit offset (xyz)
  void set_schem_we_offset(std::span<const int32_t, 3> offset) & {
    mc_schem_metadata_ir_set_we_offset(this, offset[0], offset[1], offset[2],
                                       true);
  }
  void set_schem_we_offset(const std::array<int32_t, 3>& offset) & {
    set_schem_we_offset(std::span{offset});
  }
  /// Remove the WorldEdit offset
  void set_schem_we_offset() & {
    mc_schem_metadata_ir_set_we_offset(this, 0, 0, 0, false);
  }

  /// Version of WorldEdit that created the `.schem`, or nullopt if this
  /// metadata has none
  [[nodiscard]] std::optional<std::string> schem_world_edit_version() const& {
    std::string ret;
    rust_string_receiver rsr{ret};
    const bool exist =
        mc_schem_metadata_ir_get_schem_world_edit_version(this, &rsr);
    if (exist) return ret;
    return std::nullopt;
  }
  /// Set the version of WorldEdit that created the `.schem`
  void set_schem_world_edit_version(const std::string& version) & {
    mc_schem_metadata_ir_set_schem_world_edit_version(this, version.c_str());
  }
  /// Remove the WorldEdit version
  void set_schem_world_edit_version() & {
    mc_schem_metadata_ir_set_schem_world_edit_version(this, nullptr);
  }

  /// Origin of the schematic (xyz), or nullopt if this metadata has none
  template <typename int_t = int32_t>
    requires std::is_integral_v<int_t>
  [[nodiscard]] std::optional<std::array<int_t, 3>> schem_origin() const& {
    int32_t x{0}, y{0}, z{0};
    const bool exist = mc_schem_metadata_ir_get_schem_origin(this, &x, &y, &z);
    if (exist) {
      return std::array<int_t, 3>{static_cast<int_t>(x), static_cast<int_t>(y),
                                  static_cast<int_t>(z)};
    }
    return std::nullopt;
  }
  /// Set the origin of the schematic (xyz)
  void set_schem_origin(std::span<const int32_t, 3> origin) & {
    mc_schem_metadata_ir_set_schem_origin(this, origin[0], origin[1], origin[2],
                                          true);
  }
  void set_schem_origin(const std::array<int32_t, 3>& origin) & {
    set_schem_origin(std::span{origin});
  }
  /// Remove the origin
  void set_schem_origin() & {
    mc_schem_metadata_ir_set_schem_origin(this, 0, 0, 0, false);
  }

  /// Material of the `.schem`, either `Alpha` or `Classic`
  [[nodiscard]] std::string schem_material() const& {
    std::string ret;
    rust_string_receiver rsr{ret};
    mc_schem_metadata_ir_get_schem_material(this, &rsr);
    return ret;
  }
  /// Set the material of the `.schem`, either `Alpha` or `Classic`
  void set_schem_material(const std::string& material) & {
    mc_schem_metadata_ir_set_schem_material(this, material.c_str());
  }

  /// Platform that created the `.schem`, or nullopt if this metadata has none
  [[nodiscard]] std::optional<std::string> schem_editing_platform() const& {
    std::string ret;
    rust_string_receiver rsr{ret};
    const bool exist =
        mc_schem_metadata_ir_get_schem_editing_platform(this, &rsr);
    if (exist) return ret;
    return std::nullopt;
  }
  /// Set the platform that created the `.schem`
  void set_schem_editing_platform(const std::string& platform) & {
    mc_schem_metadata_ir_set_schem_editing_platform(this, platform.c_str());
  }
  /// Remove the editing platform
  void set_schem_editing_platform() & {
    mc_schem_metadata_ir_set_schem_editing_platform(this, nullptr);
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

  /// Create an empty schematic without any region, with default metadata
  [[nodiscard]] static unique_schematic create() {
    return unique_schematic{mc_schem_create_schematic()};
  }

  [[nodiscard]] unique_schematic clone() const& {
    return unique_schematic{mc_schem_clone_schematic(this)};
  }

  void swap(schematic& another) & noexcept {
    mc_schem_swap_schematic(this, &another);
  }

  /// Metadata of this schematic
  [[nodiscard]] const metadata_ir* metadata() const& {
    return mc_schem_schematic_get_metadata(this);
  }

  /// Mutable metadata of this schematic
  [[nodiscard]] metadata_ir* metadata() & {
    return mc_schem_schematic_get_metadata_mut(this);
  }

  /// Deep copy `new_metadata` into this schematic and return the metadata that
  /// was stored before
  unique_metadata_ir set_metadata(const metadata_ir& new_metadata) & {
    auto previous_value = mc_schem_schematic_set_metadata(this, &new_metadata);
    return unique_metadata_ir{previous_value};
  }

  /// Move `new_metadata` into this schematic and return the metadata that was
  /// stored before
  unique_metadata_ir set_metadata(unique_metadata_ir new_metadata) & {
    new_metadata->swap(*this->metadata());
    return new_metadata;
  }

  /// Number of regions in this schematic
  [[nodiscard]] size_t regions_count() const& {
    return mc_schem_schematic_get_regions_count(this);
  }
  /// Region at `region_index`, or null if `region_index` is out of range
  [[nodiscard]] const region* get_region(size_t region_index) const& {
    return mc_schem_schematic_get_region(this, region_index);
  }
  /// Mutable region at `region_index`, or null if `region_index` is out of
  /// range
  [[nodiscard]] region* get_region(size_t region_index) & {
    return mc_schem_schematic_get_region_mut(this, region_index);
  }
  /// Remove and return the region at `region_index`, or null if `region_index`
  /// is out of range
  unique_region remove_region(size_t region_index) & {
    return unique_region{mc_schem_schematic_remove_region(this, region_index)};
  }
  /// Remove all regions from this schematic
  void clear_all_regions() & { mc_schem_schematic_clear_all_regions(this); }
  /// Deep copy `new_r` and insert it at `region_index`. Returns a pointer to
  /// the inserted region, or null if `region_index` is greater than
  /// regions_count()
  region* insert_region(const region* new_r, size_t region_index) & {
    return mc_schem_schematic_insert_region(this, new_r, region_index);
  }
  /// Deep copy `new_r` and append it after the last region
  region* append_region(const region* new_r) & {
    return mc_schem_schematic_insert_region(this, new_r, this->regions_count());
  }
  /// Move `new_r` into this schematic at `region_index`. Returns a pointer to
  /// the inserted region, or null if `region_index` is greater than
  /// regions_count(). `new_r` must not be used after this operation
  region* insert_region(unique_region new_r, size_t region_index) & {
    auto ret = mc_schem_schematic_insert_region_move(this, new_r.release(),
                                                     region_index);
    return ret;
  }
  /// Move `new_r` into this schematic after the last region. `new_r` must not
  /// be used after this operation
  region* append_region(unique_region new_r) & {
    return this->insert_region(std::move(new_r), this->regions_count());
  }

  /// Index of the first region containing the global position `pos`, or nullopt
  /// if no region does. "First" means the region with the lowest index, so if
  /// regions overlap the block of the lowest indexed region wins
  [[nodiscard]] std::optional<size_t> first_region_index_at(
      std::span<const int32_t, 3> pos) const& {
    const ptrdiff_t ret = mc_schem_schematic_get_first_region_index_at(
        this, pos[0], pos[1], pos[2]);
    if (ret >= 0) {
      return ret;
    }
    return std::nullopt;
  }
  [[nodiscard]] std::optional<size_t> first_region_index_at(
      const std::array<int32_t, 3>& pos) const& {
    return first_region_index_at(std::span{pos});
  }
  /// Palette index of the block at the global position `pos`, taken from the
  /// first region containing it. Returns nullopt if no region contains `pos`
  [[nodiscard]] std::optional<uint16_t> first_block_index_at(
      std::span<const int32_t, 3> pos) const& {
    bool ok = false;
    const auto ret = mc_schem_schematic_get_first_block_index_at(
        this, pos[0], pos[1], pos[2], &ok);
    if (not ok) {
      return std::nullopt;
    }
    return ret;
  }
  [[nodiscard]] std::optional<uint16_t> first_block_index_at(
      const std::array<int32_t, 3>& pos) const& {
    return first_block_index_at(std::span{pos});
  }
  /// Block at the global position `pos`, taken from the first region containing
  /// it. Returns null if no region contains `pos`
  [[nodiscard]] const block* first_block_at(
      std::span<const int32_t, 3> pos) const& {
    return mc_schem_schematic_get_first_block_at(this, pos[0], pos[1], pos[2]);
  }
  [[nodiscard]] const block* first_block_at(
      const std::array<int32_t, 3>& pos) const& {
    return first_block_at(std::span{pos});
  }
  /// Block entity at the global position `pos`, taken from the first region
  /// that has one there. Returns null if no region has one
  [[nodiscard]] const block_entity* first_block_entity_at(
      std::span<const int32_t, 3> pos) const& {
    return mc_schem_schematic_get_first_block_entity_at(this, pos[0], pos[1],
                                                        pos[2]);
  }
  [[nodiscard]] const block_entity* first_block_entity_at(
      const std::array<int32_t, 3>& pos) const& {
    return first_block_entity_at(std::span{pos});
  }
  /// Pending ticks at the global position `pos`, taken from the first region
  /// that has any there. Returns an empty vector if no region has one
  [[nodiscard]] std::vector<const pending_tick*> first_pending_ticks_at(
      std::span<const int32_t, 3> pos) const& {
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
  [[nodiscard]] std::vector<const pending_tick*> first_pending_ticks_at(
      const std::array<int32_t, 3>& pos) const& {
    return first_pending_ticks_at(std::span{pos});
  }

  /// Enclosing shape (xyz) of all regions. Returns {0, 0, 0} if this schematic
  /// has no region
  template <typename int_t = int32_t>
    requires std::is_integral_v<int_t>
  [[nodiscard]] std::array<int_t, 3> shape() const& {
    int32_t x = -1, y = -1, z = -1;
    mc_schem_schematic_get_shape(this, &x, &y, &z);
    return {static_cast<int_t>(x), static_cast<int_t>(y),
            static_cast<int_t>(z)};
  }
  /// Volume of the enclosing box of all regions
  [[nodiscard]] uint64_t volume() const& {
    return mc_schem_schematic_get_volume(this);
  }
  /// Count of blocks over all regions. Air is counted only when `include_air`
  /// is true
  [[nodiscard]] uint64_t total_blocks(bool include_air) const& {
    return mc_schem_schematic_get_total_blocks(this, include_air);
  }
  /// Merge all regions into one region without changing this schematic. Space
  /// that no region covers is filled with `background_block`
  [[nodiscard]] unique_region to_single_region(
      const block& background_block) const& {
    return unique_region{
        mc_schem_schematic_to_single_region(this, &background_block)};
  }
  /// Merge all regions in place, leaving exactly one region. Space that no
  /// region covers is filled with `background_block`
  void merge_regions(const block& background_block) & {
    mc_schem_schematic_merge_regions(this, &background_block);
  }
  /// Load a litematica (*.litematic) schematic from `is`. The stream raw nbt
  /// unless `opt.is_compressed` is false. Returns the schematic together with
  /// its raw litematica metadata
  [[nodiscard]] static std::expected<
      std::tuple<unique_schematic, unique_litematica_metadata>, shared_error>
  load_litematica(std::istream& is, const litematica_load_option& opt) {
    error* err = nullptr;
    litematica_metadata* raw_md{nullptr};
    rust_reader isw{is};
    auto ret = mc_schem_schematic_load_litematica_from_reader(&isw, &opt, &err,
                                                              &raw_md);
    if (ret) {
      assert(err == nullptr);
      assert(raw_md not_eq nullptr);
      return std::make_tuple(unique_schematic{ret},
                             unique_litematica_metadata{raw_md});
    }
    assert(err not_eq nullptr);
    return std::unexpected(std::shared_ptr{unique_error{err}});
  }
  /// Load a vanilla structure (*.nbt) schematic from `is`. The stream is
  /// raw nbt unless `opt.is_compressed` is false. Returns the schematic
  /// together with its raw vanilla structure metadata
  [[nodiscard]] static std::expected<
      std::tuple<unique_schematic, unique_vanilla_structure_metadata>,
      shared_error>
  load_vanilla_structure(std::istream& is,
                         const vanilla_structure_load_option& opt) {
    error* err = nullptr;
    vanilla_structure_metadata* raw_md{nullptr};
    rust_reader isw{is};
    auto ret = mc_schem_schematic_load_vanilla_structure_from_reader(
        &isw, &opt, &err, &raw_md);
    if (ret) {
      assert(err == nullptr);
      assert(raw_md not_eq nullptr);
      return std::make_tuple(unique_schematic{ret},
                             unique_vanilla_structure_metadata{raw_md});
    }
    assert(err not_eq nullptr);
    return std::unexpected(std::shared_ptr{unique_error{err}});
  }
  /// Load a WorldEdit 1.13+ (*.schem) schematic from `is`. The stream is
  /// raw nbt unless `opt.is_compressed` is false. Returns the schematic
  /// together with its raw WorldEdit metadata
  [[nodiscard]] static std::expected<
      std::tuple<unique_schematic, unique_world_edit13_metadata>, shared_error>
  load_world_edit13(std::istream& is, const world_edit13_load_option& opt) {
    error* err = nullptr;
    world_edit13_metadata* raw_md{nullptr};
    rust_reader isw{is};
    auto ret = mc_schem_schematic_load_world_edit13_from_reader(&isw, &opt,
                                                                &err, &raw_md);
    if (ret) {
      assert(err == nullptr);
      assert(raw_md not_eq nullptr);
      return std::make_tuple(unique_schematic{ret},
                             unique_world_edit13_metadata{raw_md});
    }
    assert(err not_eq nullptr);
    return std::unexpected(std::shared_ptr{unique_error{err}});
  }

  /// Load a WorldEdit 1.12 (*.schematic) schematic from `is`. The stream is
  /// raw nbt unless `opt.is_compressed` is false. Returns the schematic
  /// together with its raw WorldEdit metadata
  [[nodiscard]] static std::expected<
      std::tuple<unique_schematic, unique_world_edit12_metadata>, shared_error>
  load_world_edit12(std::istream& is, const world_edit12_load_option& opt) {
    error* err = nullptr;
    world_edit12_metadata* raw_md{nullptr};
    rust_reader isw{is};
    auto ret = mc_schem_schematic_load_world_edit12_from_reader(&isw, &opt,
                                                                &err, &raw_md);
    if (ret) {
      assert(err == nullptr);
      assert(raw_md not_eq nullptr);
      return std::make_tuple(unique_schematic{ret},
                             unique_world_edit12_metadata{raw_md});
    }
    assert(err not_eq nullptr);
    return std::unexpected(std::shared_ptr{unique_error{err}});
  }
  /// Load a schematic from a file, identifying the format by the filename
  /// extension. Decompression is handled internally and the raw per-format
  /// metadata is not exposed
  [[nodiscard]] static std::expected<unique_schematic, shared_error>
  load_from_file(const char* filename) {
    error* err = nullptr;
    auto schem = mc_schem_schematic_load_from_file(filename, &err);
    if (schem) {
      assert(err == nullptr);
      return unique_schematic{schem};
    }
    assert(err not_eq nullptr);
    return std::unexpected(std::shared_ptr{unique_error{err}});
  }
  /// Save as litematica (*.litematic). The bytes written to `os` are gzip
  /// compressed
  std::expected<void, shared_error> save_litematica(
      std::ostream& os, const litematica_save_option& opt) const& {
    rust_writer osw{os};
    auto err = mc_schem_schematic_save_litematica_to_writer(this, &osw, &opt);
    if (not err) {
      return {};
    }
    return std::unexpected(std::shared_ptr{unique_error{err}});
  }
  /// Save as WorldEdit 1.13+ (*.schem). The bytes written to `os` are gzip
  /// compressed
  std::expected<void, shared_error> save_world_edit13(
      std::ostream& os, const world_edit13_save_option& opt) const& {
    rust_writer osw{os};
    auto err = mc_schem_schematic_save_world_edit13_to_writer(this, &osw, &opt);
    if (not err) {
      return {};
    }
    return std::unexpected(std::shared_ptr{unique_error{err}});
  }
  /// Save to vanilla structure (*.nbt). The bytes written to `os` are gzip
  /// compressed
  std::expected<void, shared_error> save_vanilla_structure(
      std::ostream& os, const vanilla_structure_save_option& opt) const& {
    rust_writer osw{os};
    auto err =
        mc_schem_schematic_save_vanilla_structure_to_writer(this, &osw, &opt);
    if (not err) {
      return {};
    }
    return std::unexpected(std::shared_ptr{unique_error{err}});
  }
  /// Save this schematic to a file, identifying the format by the filename
  /// extension. Compression is handled inside
  std::expected<void, shared_error> save_to_file(const char* filename) const& {
    auto error = mc_schem_schematic_save_to_file(this, filename);
    if (not error) {
      return {};
    }
    return std::unexpected(unique_error{error});
  }
};

}  // namespace mc_schem

#endif  // MC_SCHEM_MC_SCHEM_HPP
