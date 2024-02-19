#ifndef MC_SCHEM_H
#define MC_SCHEM_H

#include <mc_schem_export.h>

#ifndef __cplusplus

#include <stdint.h>
#include <stdbool.h>
#include <stddef.h>
#include <string.h>
#include <stdio.h>
#include <stdalign.h>

#else

#include <cstdint>
#include <cstdbool>
#include <cstddef>
#include <cstring>
#include <cstdio>
#include <cstdalign>
#endif

#define MC_SCHEM_DEFINE_BOX(content_type) \
typedef struct {                          \
  content_type * ptr;                       \
} content_type##_box;


#define MC_SCHEM_DEFINE_OPTIONAL(type, name) \
typedef struct {                       \
  type value;                          \
  bool has_value;                   \
}MC_SCHEM_optional_##name;


#ifdef __cplusplus
extern "C" {
#endif


MC_SCHEM_EXPORT const char *MC_SCHEM_version_string();

MC_SCHEM_EXPORT uint16_t MC_SCHEM_version_major();

MC_SCHEM_EXPORT uint16_t MC_SCHEM_version_minor();

MC_SCHEM_EXPORT uint16_t MC_SCHEM_version_patch();

MC_SCHEM_EXPORT uint16_t MC_SCHEM_version_tweak();

/////////////////////////////////////////////

typedef struct {
  int pos[3];
} MC_SCHEM_array3_i32;
typedef struct {
  double pos[3];
} MC_SCHEM_array3_f64;

typedef struct {
  const char *begin;
  const char *end;
} MC_SCHEM_string_view;

// Rust String
typedef struct MC_SCHEM_string_s MC_SCHEM_string;
// Box<String> in rust
MC_SCHEM_DEFINE_BOX(MC_SCHEM_string)

MC_SCHEM_EXPORT MC_SCHEM_string_view MC_SCHEM_string_unwrap(const MC_SCHEM_string *);
// Set rust string
MC_SCHEM_EXPORT void MC_SCHEM_string_set(MC_SCHEM_string *, MC_SCHEM_string_view src);

MC_SCHEM_string_view MC_SCHEM_c_string_to_string_view(const char *str);

//////////////////////////////////

typedef struct {
  size_t reserved[7];
} MC_SCHEM_nbt_value;
typedef struct MC_SCHEM_block_s {
  size_t reserved[9];
} MC_SCHEM_block;
MC_SCHEM_DEFINE_BOX(MC_SCHEM_block)
typedef struct MC_SCHEM_block_entity_s {
  size_t reserved[11];
} MC_SCHEM_block_entity;
MC_SCHEM_DEFINE_BOX(MC_SCHEM_block_entity)
typedef struct MC_SCHEM_pending_tick_s MC_SCHEM_pending_tick;
MC_SCHEM_DEFINE_BOX(MC_SCHEM_pending_tick)

typedef enum : uint8_t {
  MC_SCHEM_MKT_string = 0,
  MC_SCHEM_MKT_pos_i32 = 1,
} MC_SCHEM_map_key_type;

typedef enum : uint8_t {
  MC_SCHEM_MVT_string = 0,
  MC_SCHEM_MVT_nbt = 1,
  MC_SCHEM_MVT_block_entity = 2,
  MC_SCHEM_MVT_pending_tick = 3,
} MC_SCHEM_map_value_type;

typedef struct {
  size_t reserved[2];
} MC_SCHEM_map_ref;//typed pointer to a BTreeMap/HashMap

typedef struct {
  size_t reserved[2];
} MC_SCHEM_map_box;

typedef struct {
  size_t reserved[12];
} MC_SCHEM_map_iterator;

typedef union {
  MC_SCHEM_string_view string;
  int pos[3];
} MC_SCHEM_key_wrapper;

typedef union {
  MC_SCHEM_string *string;
  MC_SCHEM_nbt_value *nbt;
  MC_SCHEM_block_entity *block_entity;
  MC_SCHEM_pending_tick *pending_tick;
} MC_SCHEM_value_wrapper;

MC_SCHEM_EXPORT MC_SCHEM_map_ref MC_SCHEM_map_unwrap_box(const MC_SCHEM_map_box *);

MC_SCHEM_EXPORT MC_SCHEM_map_key_type MC_SCHEM_map_get_key_type(const MC_SCHEM_map_ref *);

MC_SCHEM_EXPORT MC_SCHEM_map_value_type MC_SCHEM_map_get_value_type(const MC_SCHEM_map_ref *);

MC_SCHEM_EXPORT MC_SCHEM_map_box
MC_SCHEM_create_map(MC_SCHEM_map_key_type key_t, MC_SCHEM_map_value_type val_t, bool *ok);

MC_SCHEM_EXPORT void MC_SCHEM_release_map(MC_SCHEM_map_box *box);

MC_SCHEM_EXPORT MC_SCHEM_value_wrapper
MC_SCHEM_map_find(const MC_SCHEM_map_ref *map,
                  MC_SCHEM_map_key_type key_t,
                  MC_SCHEM_map_value_type val_t,
                  const MC_SCHEM_key_wrapper *key, bool *ok);

bool MC_SCHEM_map_contains_key(const MC_SCHEM_map_ref *map,
                               MC_SCHEM_map_key_type key_t,
                               const MC_SCHEM_key_wrapper *key);

MC_SCHEM_EXPORT size_t MC_SCHEM_map_length(const MC_SCHEM_map_ref *map);

MC_SCHEM_EXPORT size_t MC_SCHEM_map_capacity(const MC_SCHEM_map_ref *map);

MC_SCHEM_EXPORT void MC_SCHEM_map_reserve(MC_SCHEM_map_ref *map, size_t new_capacity);

MC_SCHEM_EXPORT void MC_SCHEM_map_insert(MC_SCHEM_map_ref *map, MC_SCHEM_key_wrapper key,
                                         MC_SCHEM_value_wrapper value);

MC_SCHEM_EXPORT void MC_SCHEM_map_remove(MC_SCHEM_map_ref *map, MC_SCHEM_key_wrapper key,
                                         bool *has_value_before_erase_nullable);

MC_SCHEM_EXPORT void MC_SCHEM_map_foreach(const MC_SCHEM_map_ref *map,
                                          void(*fun)(size_t index,
                                                     MC_SCHEM_key_wrapper key,
                                                     MC_SCHEM_value_wrapper value,
                                                     void *custom_data),
                                          void *custom_data);

MC_SCHEM_EXPORT MC_SCHEM_map_iterator
MC_SCHEM_map_iterator_first(const MC_SCHEM_map_ref *map,
                            MC_SCHEM_map_key_type key_t,
                            MC_SCHEM_map_value_type val_t,
                            bool *ok);

MC_SCHEM_EXPORT MC_SCHEM_map_iterator MC_SCHEM_map_iterator_end(const MC_SCHEM_map_ref *map,
                                                                MC_SCHEM_map_key_type key_t,
                                                                MC_SCHEM_map_value_type val_t,
                                                                bool *ok);

//
//MC_SCHEM_EXPORT MC_SCHEM_iterator_next_result
//MC_SCHEM_map_iterator_next(MC_SCHEM_map_iterator *it);

MC_SCHEM_EXPORT void MC_SCHEM_map_iterator_add(MC_SCHEM_map_iterator *it);

typedef struct {
  MC_SCHEM_key_wrapper key;
  MC_SCHEM_value_wrapper value;
  bool has_value;
} MC_SCHEM_iterator_deref_result;
MC_SCHEM_EXPORT MC_SCHEM_iterator_deref_result MC_SCHEM_map_iterator_deref(const MC_SCHEM_map_iterator *it);

MC_SCHEM_EXPORT size_t MC_SCHEM_map_iterator_length(const MC_SCHEM_map_iterator *it);

MC_SCHEM_EXPORT bool MC_SCHEM_map_iterator_equal(const MC_SCHEM_map_iterator *a,
                                                 const MC_SCHEM_map_iterator *b);

bool MC_SCHEM_map_iterator_is_end(const MC_SCHEM_map_iterator *it);

//////////////////////////////////////////
// NBT APIs

typedef enum : uint8_t {
  MC_SCHEM_nbt_type_byte = 1,
  MC_SCHEM_nbt_type_short = 2,
  MC_SCHEM_nbt_type_int = 3,
  MC_SCHEM_nbt_type_long = 4,
  MC_SCHEM_nbt_type_float = 5,
  MC_SCHEM_nbt_type_double = 6,
  MC_SCHEM_nbt_type_byte_array = 7,
  MC_SCHEM_nbt_type_string = 8,
  MC_SCHEM_nbt_type_list = 9,
  MC_SCHEM_nbt_type_compound = 10,
  MC_SCHEM_nbt_type_int_array = 11,
  MC_SCHEM_nbt_type_long_array = 12,
} MC_SCHEM_nbt_type;

MC_SCHEM_DEFINE_BOX(MC_SCHEM_nbt_value)

MC_SCHEM_EXPORT MC_SCHEM_nbt_value_box MC_SCHEM_create_nbt();

MC_SCHEM_EXPORT void MC_SCHEM_release_nbt(MC_SCHEM_nbt_value_box *nbt_box);

MC_SCHEM_EXPORT MC_SCHEM_nbt_type MC_SCHEM_nbt_get_type(const MC_SCHEM_nbt_value *);

MC_SCHEM_EXPORT int8_t MC_SCHEM_nbt_get_byte(const MC_SCHEM_nbt_value *, bool *ok);

MC_SCHEM_EXPORT void MC_SCHEM_nbt_set_byte(MC_SCHEM_nbt_value *, int8_t);

MC_SCHEM_EXPORT int16_t MC_SCHEM_nbt_get_short(const MC_SCHEM_nbt_value *, bool *ok);

MC_SCHEM_EXPORT void MC_SCHEM_nbt_set_short(MC_SCHEM_nbt_value *, int16_t);

MC_SCHEM_EXPORT int32_t MC_SCHEM_nbt_get_int(const MC_SCHEM_nbt_value *, bool *ok);

MC_SCHEM_EXPORT void MC_SCHEM_nbt_set_int(MC_SCHEM_nbt_value *, int32_t);

MC_SCHEM_EXPORT int64_t MC_SCHEM_nbt_get_long(const MC_SCHEM_nbt_value *, bool *ok);

MC_SCHEM_EXPORT void MC_SCHEM_nbt_set_long(MC_SCHEM_nbt_value *, int64_t);

MC_SCHEM_EXPORT float MC_SCHEM_nbt_get_float(const MC_SCHEM_nbt_value *, bool *ok);

MC_SCHEM_EXPORT void MC_SCHEM_nbt_set_float(MC_SCHEM_nbt_value *, float);

MC_SCHEM_EXPORT double MC_SCHEM_nbt_get_double(const MC_SCHEM_nbt_value *, bool *ok);

MC_SCHEM_EXPORT void MC_SCHEM_nbt_set_double(MC_SCHEM_nbt_value *, double);

MC_SCHEM_EXPORT MC_SCHEM_string *
MC_SCHEM_nbt_get_string(const MC_SCHEM_nbt_value *, bool *ok);

MC_SCHEM_EXPORT void MC_SCHEM_nbt_set_string(MC_SCHEM_nbt_value *, MC_SCHEM_string_view);

typedef struct {
  int8_t *begin;
  int8_t *end;
} MC_SCHEM_nbt_byte_array_view;
MC_SCHEM_EXPORT MC_SCHEM_nbt_byte_array_view MC_SCHEM_nbt_get_byte_array(const MC_SCHEM_nbt_value *, bool *ok);

MC_SCHEM_EXPORT void MC_SCHEM_nbt_set_byte_array(MC_SCHEM_nbt_value *, MC_SCHEM_nbt_byte_array_view);

typedef struct {
  int32_t *begin;
  int32_t *end;
} MC_SCHEM_nbt_int_array_view;
MC_SCHEM_EXPORT MC_SCHEM_nbt_int_array_view MC_SCHEM_nbt_get_int_array(const MC_SCHEM_nbt_value *, bool *ok);

MC_SCHEM_EXPORT void MC_SCHEM_nbt_set_int_array(MC_SCHEM_nbt_value *, MC_SCHEM_nbt_int_array_view);

typedef struct {
  int64_t *begin;
  int64_t *end;
} MC_SCHEM_nbt_long_array_view;
MC_SCHEM_EXPORT MC_SCHEM_nbt_long_array_view MC_SCHEM_nbt_get_long_array(const MC_SCHEM_nbt_value *, bool *ok);

MC_SCHEM_EXPORT void MC_SCHEM_nbt_set_long_array(MC_SCHEM_nbt_value *, MC_SCHEM_nbt_long_array_view);

typedef struct {
  MC_SCHEM_nbt_value *begin;
  MC_SCHEM_nbt_value *end;
} MC_SCHEM_nbt_list_view;
MC_SCHEM_EXPORT MC_SCHEM_nbt_list_view MC_SCHEM_nbt_get_list(const MC_SCHEM_nbt_value *, bool *ok);

MC_SCHEM_EXPORT void MC_SCHEM_nbt_set_list(MC_SCHEM_nbt_value *, MC_SCHEM_nbt_list_view);

MC_SCHEM_EXPORT MC_SCHEM_map_ref MC_SCHEM_nbt_get_compound(const MC_SCHEM_nbt_value *, bool *ok);

MC_SCHEM_EXPORT void MC_SCHEM_nbt_set_compound(MC_SCHEM_nbt_value *, MC_SCHEM_map_ref, bool *ok);

////////////////////////////////////////
// block related APIs

MC_SCHEM_EXPORT MC_SCHEM_block_box MC_SCHEM_create_block();

MC_SCHEM_EXPORT void MC_SCHEM_release_block(MC_SCHEM_block_box *);

MC_SCHEM_EXPORT MC_SCHEM_string_view MC_SCHEM_block_get_namespace(const MC_SCHEM_block *);

MC_SCHEM_EXPORT MC_SCHEM_string_view MC_SCHEM_block_get_id(const MC_SCHEM_block *);

MC_SCHEM_EXPORT MC_SCHEM_map_ref MC_SCHEM_block_get_attributes(const MC_SCHEM_block *);

MC_SCHEM_EXPORT void MC_SCHEM_block_set_namespace(MC_SCHEM_block *, MC_SCHEM_string_view namespace_);

MC_SCHEM_EXPORT void MC_SCHEM_block_set_id(MC_SCHEM_block *, MC_SCHEM_string_view id);

MC_SCHEM_EXPORT void MC_SCHEM_block_set_attributes(MC_SCHEM_block *, MC_SCHEM_map_ref map, bool *ok);

typedef enum : uint8_t {
  MC_SCHEM_BIPE_too_many_colons = 0,
  MC_SCHEM_BIPE_too_many_left_brackets = 1,
  MC_SCHEM_BIPE_too_many_right_brackets = 2,
  MC_SCHEM_BIPE_missing_block_id = 3,
  MC_SCHEM_BIPE_brackets_not_in_pairs = 4,
  MC_SCHEM_BIPE_bracket_in_wrong_position = 5,
  MC_SCHEM_BIPE_colons_in_wrong_position = 6,
  MC_SCHEM_BIPE_missing_equal_in_attributes = 7,
  MC_SCHEM_BIPE_too_many_equals_in_attributes = 8,
  MC_SCHEM_BIPE_missing_attribute_name = 9,
  MC_SCHEM_BIPE_missing_attribute_value = 10,
  MC_SCHEM_BIPE_extra_string_after_right_bracket = 11,
  MC_SCHEM_BIPE_invalid_character = 12,
} MC_SCHEM_block_id_parse_error;

MC_SCHEM_EXPORT bool
MC_SCHEM_parse_block(MC_SCHEM_string_view full_id, MC_SCHEM_block *dest, MC_SCHEM_block_id_parse_error *error_nullable);

MC_SCHEM_EXPORT void
MC_SCHEM_block_to_full_id(const MC_SCHEM_block *block, char *id_dest_nullable, size_t capacity, size_t *id_length);

////////////////////////
typedef enum : uint16_t {
  MC_SCHEM_common_block_air = 0,
  MC_SCHEM_common_block_structure_void = 1,
} MC_SCHEM_common_block;
////////////////////////////////////////

typedef struct MC_SCHEM_entity_s MC_SCHEM_entity;
MC_SCHEM_DEFINE_BOX(MC_SCHEM_entity)

MC_SCHEM_EXPORT MC_SCHEM_entity_box MC_SCHEM_create_entity();

MC_SCHEM_EXPORT void MC_SCHEM_release_entity(MC_SCHEM_entity_box *);

MC_SCHEM_EXPORT MC_SCHEM_array3_i32 MC_SCHEM_entity_get_block_pos(const MC_SCHEM_entity *);

MC_SCHEM_EXPORT MC_SCHEM_array3_f64 MC_SCHEM_entity_get_pos(const MC_SCHEM_entity *);

MC_SCHEM_EXPORT void MC_SCHEM_entity_set_block_pos(MC_SCHEM_entity *, MC_SCHEM_array3_i32);

MC_SCHEM_EXPORT void MC_SCHEM_entity_set_pos(MC_SCHEM_entity *, MC_SCHEM_array3_f64);


MC_SCHEM_EXPORT MC_SCHEM_map_ref MC_SCHEM_entity_get_tags(const MC_SCHEM_entity *);

//////////////////////////////////

MC_SCHEM_EXPORT MC_SCHEM_block_entity_box MC_SCHEM_create_block_entity();

MC_SCHEM_EXPORT void MC_SCHEM_release_block_entity(MC_SCHEM_block_entity_box *);

MC_SCHEM_EXPORT MC_SCHEM_map_ref MC_SCHEM_block_entity_get_tags(const MC_SCHEM_block_entity *);

//////////////////////////////////

MC_SCHEM_EXPORT MC_SCHEM_pending_tick_box MC_SCHEM_create_pending_tick();

MC_SCHEM_EXPORT void MC_SCHEM_release_pending_tick(MC_SCHEM_pending_tick_box *);

MC_SCHEM_EXPORT int32_t MC_SCHEM_pending_tick_get_priority(const MC_SCHEM_pending_tick *);

MC_SCHEM_EXPORT void MC_SCHEM_pending_tick_set_priority(MC_SCHEM_pending_tick *, int32_t priority);

MC_SCHEM_EXPORT int64_t MC_SCHEM_pending_tick_get_sub_tick(const MC_SCHEM_pending_tick *);

MC_SCHEM_EXPORT void MC_SCHEM_pending_tick_set_sub_tick(MC_SCHEM_pending_tick *, int64_t sub_tick);

MC_SCHEM_EXPORT int32_t MC_SCHEM_pending_tick_get_time(const MC_SCHEM_pending_tick *);

MC_SCHEM_EXPORT void MC_SCHEM_pending_tick_set_time(MC_SCHEM_pending_tick *, int32_t time);

typedef enum : uint8_t {
  MC_SCHME_BET_fluid = 0,
  MC_SCHEM_BET_block = 1,
} MC_SCHEM_pending_tick_type;
MC_SCHEM_EXPORT MC_SCHEM_string_view MC_SCHEM_pending_tick_get_id(const MC_SCHEM_pending_tick *);

MC_SCHEM_EXPORT MC_SCHEM_pending_tick_type MC_SCHEM_pending_tick_get_type(const MC_SCHEM_pending_tick *);

MC_SCHEM_EXPORT void MC_SCHEM_pending_tick_set_info(MC_SCHEM_pending_tick *,
                                                    MC_SCHEM_pending_tick_type type,
                                                    MC_SCHEM_string_view id);

//////////////////////////////////
typedef struct MC_SCHEM_error_s MC_SCHEM_error;
MC_SCHEM_DEFINE_BOX(MC_SCHEM_error)

MC_SCHEM_EXPORT void MC_SCHEM_release_error(MC_SCHEM_error_box *);

MC_SCHEM_EXPORT void MC_SCHEM_error_to_string(const MC_SCHEM_error *, char *dest, size_t capacity, size_t *length);

MC_SCHEM_EXPORT MC_SCHEM_error_box MC_SCHEM_error_test_none();

MC_SCHEM_EXPORT MC_SCHEM_error_box MC_SCHEM_error_test_some();

//////////////////////////////////
typedef struct MC_SCHEM_region_s MC_SCHEM_region;
MC_SCHEM_DEFINE_BOX(MC_SCHEM_region)

MC_SCHEM_EXPORT MC_SCHEM_region_box MC_SCHEM_create_region();

MC_SCHEM_EXPORT void MC_SCHEM_release_region(MC_SCHEM_region_box *);

MC_SCHEM_EXPORT MC_SCHEM_string_view MC_SCHEM_region_get_name(const MC_SCHEM_region *);

MC_SCHEM_EXPORT void MC_SCHEM_region_set_name(MC_SCHEM_region *, MC_SCHEM_string_view name);

MC_SCHEM_EXPORT MC_SCHEM_array3_i32 MC_SCHEM_region_get_offset(const MC_SCHEM_region *);

MC_SCHEM_EXPORT void MC_SCHEM_region_set_offset(MC_SCHEM_region *, MC_SCHEM_array3_i32 offset);

MC_SCHEM_EXPORT MC_SCHEM_block *MC_SCHEM_region_get_palette(const MC_SCHEM_region *, size_t *len);

MC_SCHEM_EXPORT void MC_SCHEM_region_set_palette(MC_SCHEM_region *, const MC_SCHEM_block *const *palette, size_t len);

MC_SCHEM_EXPORT MC_SCHEM_map_ref MC_SCHEM_region_get_block_entities(const MC_SCHEM_region *);

MC_SCHEM_EXPORT MC_SCHEM_map_ref MC_SCHEM_region_get_pending_ticks(const MC_SCHEM_region *);

MC_SCHEM_EXPORT MC_SCHEM_entity *MC_SCHEM_region_get_entities(const MC_SCHEM_region *, size_t *len);

MC_SCHEM_EXPORT uint16_t *MC_SCHEM_region_get_block_index_array(const MC_SCHEM_region *);

typedef struct {
  uint8_t id;
  uint8_t damage;
} MC_SCHEM_number_id;
MC_SCHEM_EXPORT MC_SCHEM_number_id *
MC_SCHEM_region_get_number_id_array(const MC_SCHEM_region *);

MC_SCHEM_EXPORT MC_SCHEM_array3_i32 MC_SCHEM_region_get_shape(const MC_SCHEM_region *);

MC_SCHEM_EXPORT void MC_SCHEM_region_reshape(MC_SCHEM_region *, MC_SCHEM_array3_i32);

MC_SCHEM_EXPORT const MC_SCHEM_block *
MC_SCHEM_region_get_block(const MC_SCHEM_region *, MC_SCHEM_array3_i32 r_pos);

MC_SCHEM_EXPORT bool MC_SCHEM_region_set_block(MC_SCHEM_region *, MC_SCHEM_array3_i32 r_pos, const MC_SCHEM_block *);

MC_SCHEM_EXPORT uint16_t MC_SCHEM_region_get_block_index(const MC_SCHEM_region *, MC_SCHEM_array3_i32 r_pos);

MC_SCHEM_EXPORT bool
MC_SCHEM_region_set_block_index(MC_SCHEM_region *, MC_SCHEM_array3_i32 r_pos, uint16_t block_index);

MC_SCHEM_EXPORT uint64_t MC_SCHEM_region_get_volume(const MC_SCHEM_region *);

MC_SCHEM_EXPORT uint64_t MC_SCHEM_region_get_total_blocks(const MC_SCHEM_region *, bool include_air);

MC_SCHEM_EXPORT uint16_t MC_SCHEM_region_get_block_index_of_air(const MC_SCHEM_region *, bool *ok);

MC_SCHEM_EXPORT uint16_t MC_SCHEM_region_get_block_index_of_structure_void(const MC_SCHEM_region *, bool *ok);

MC_SCHEM_EXPORT bool MC_SCHEM_region_contains_coordinate(const MC_SCHEM_region *, MC_SCHEM_array3_i32 r_pos);

typedef struct {
  uint16_t block_index;
  const MC_SCHEM_block *block;
  MC_SCHEM_block_entity *block_entity;
  MC_SCHEM_pending_tick *pending_tick;
} MC_SCHEM_region_block_info;
MC_SCHEM_EXPORT MC_SCHEM_region_block_info
MC_SCHEM_region_get_block_info(const MC_SCHEM_region *, MC_SCHEM_array3_i32 r_pos);

// box.ptr will contain NULL if no error
MC_SCHEM_EXPORT MC_SCHEM_error_box MC_SCHEM_region_shrink_palette(MC_SCHEM_region *);

//////////////////////////////////
typedef struct MC_SCHEM_schem_s MC_SCHEM_schematic;
MC_SCHEM_DEFINE_BOX(MC_SCHEM_schematic)

MC_SCHEM_EXPORT MC_SCHEM_schematic_box MC_SCHEM_create_schem();

MC_SCHEM_EXPORT void MC_SCHEM_release_schem(MC_SCHEM_schematic_box *);


typedef struct {
  void *handle;

  size_t (*read_fun)(void *handle, uint8_t *buffer, size_t buffer_size,
                     bool *ok, char *error, size_t error_capacity);
} MC_SCHEM_reader;

MC_SCHEM_reader MC_SCHEM_reader_wrap_stream(FILE *);

typedef struct {
  MC_SCHEM_schematic_box schematic;
  MC_SCHEM_error_box error;
} MC_SCHEM_schem_load_result;

typedef struct {
  alignas(512) uint8_t reserved[512];
} MC_SCHEM_load_option_litematica;
static_assert(sizeof(MC_SCHEM_load_option_litematica)==512);
MC_SCHEM_EXPORT MC_SCHEM_load_option_litematica MC_SCHEM_load_option_litematica_default();

MC_SCHEM_EXPORT MC_SCHEM_schem_load_result
MC_SCHEM_schem_load_litematica(MC_SCHEM_reader src, const MC_SCHEM_load_option_litematica *option);

MC_SCHEM_EXPORT MC_SCHEM_schem_load_result
MC_SCHEM_schem_load_litematica_file(MC_SCHEM_string_view filename, const MC_SCHEM_load_option_litematica *option);

MC_SCHEM_EXPORT MC_SCHEM_schem_load_result
MC_SCHEM_schem_load_litematica_bytes(const uint8_t *bytes, size_t length,
                                     const MC_SCHEM_load_option_litematica *option);

typedef struct {
  alignas(512) MC_SCHEM_common_block background_block;
} MC_SCHEM_load_option_vanilla_structure;
static_assert(sizeof(MC_SCHEM_load_option_vanilla_structure)==512,"");
MC_SCHEM_EXPORT MC_SCHEM_load_option_vanilla_structure MC_SCHEM_load_option_vanilla_structure_default();

MC_SCHEM_EXPORT MC_SCHEM_schem_load_result
MC_SCHEM_schem_load_vanilla_structure(MC_SCHEM_reader src, const MC_SCHEM_load_option_vanilla_structure *option);

MC_SCHEM_EXPORT MC_SCHEM_schem_load_result
MC_SCHEM_schem_load_vanilla_structure_file(MC_SCHEM_string_view filename,
                                           const MC_SCHEM_load_option_vanilla_structure *option);

MC_SCHEM_EXPORT MC_SCHEM_schem_load_result
MC_SCHEM_schem_load_vanilla_structure_bytes(const uint8_t *bytes, size_t length,
                                            const MC_SCHEM_load_option_vanilla_structure *option);

typedef struct {
  alignas(512) uint8_t reserved[512];
} MC_SCHEM_load_option_world_edit_13;
static_assert(sizeof(MC_SCHEM_load_option_world_edit_13)==512,"");
MC_SCHEM_EXPORT MC_SCHEM_load_option_world_edit_13 MC_SCHEM_load_option_world_edit_13_default();

MC_SCHEM_EXPORT MC_SCHEM_schem_load_result
MC_SCHEM_schem_load_world_edit_13(MC_SCHEM_reader src, const MC_SCHEM_load_option_world_edit_13 *option);

MC_SCHEM_EXPORT MC_SCHEM_schem_load_result
MC_SCHEM_schem_load_world_edit_13_file(MC_SCHEM_string_view filename, const MC_SCHEM_load_option_world_edit_13 *option);

MC_SCHEM_EXPORT MC_SCHEM_schem_load_result
MC_SCHEM_schem_load_world_edit_13_bytes(const uint8_t *bytes, size_t length,
                                        const MC_SCHEM_load_option_world_edit_13 *option);

typedef struct {
  alignas(512) int32_t data_version;
  bool fix_string_id_with_block_entity_data;
  bool discard_number_id_array;
} MC_SCHEM_load_option_world_edit_12;
static_assert(sizeof(MC_SCHEM_load_option_world_edit_12)==512,"");
MC_SCHEM_EXPORT MC_SCHEM_load_option_world_edit_12 MC_SCHEM_load_option_world_edit_12_default();

MC_SCHEM_EXPORT MC_SCHEM_schem_load_result
MC_SCHEM_schem_load_world_edit_12(MC_SCHEM_reader src, const MC_SCHEM_load_option_world_edit_12 *option);

MC_SCHEM_EXPORT MC_SCHEM_schem_load_result
MC_SCHEM_schem_load_world_edit_12_file(MC_SCHEM_string_view filename, const MC_SCHEM_load_option_world_edit_12 *option);

MC_SCHEM_EXPORT MC_SCHEM_schem_load_result
MC_SCHEM_schem_load_world_edit_12_bytes(const uint8_t *bytes, size_t length,
                                        const MC_SCHEM_load_option_world_edit_12 *option);


typedef struct {
  void *handle;

  size_t (*write_fun)(void *handle, const uint8_t *buf, size_t buf_size, bool *ok, char *error, size_t error_capacity);

  void (*flush_fun)(void *handle, bool *ok, char *error, size_t error_capacity);
} MC_SCHEM_writer;

MC_SCHEM_writer MC_SCHEM_writer_wrap_stream(FILE *f);

typedef struct {
  alignas(512) uint32_t compress_level;
  bool rename_duplicated_regions;
  //uint8_t reserved[507];
} MC_SCHEM_save_option_litematica;
static_assert(sizeof(MC_SCHEM_save_option_litematica)==512,"");
MC_SCHEM_EXPORT MC_SCHEM_save_option_litematica MC_SCHEM_save_option_litematica_default();
MC_SCHEM_EXPORT MC_SCHEM_error_box
MC_SCHEM_schem_save_litematica(const MC_SCHEM_schematic *, MC_SCHEM_writer writer,
                               const MC_SCHEM_save_option_litematica *option);

MC_SCHEM_EXPORT MC_SCHEM_error_box
MC_SCHEM_schem_save_litematica_file(const MC_SCHEM_schematic *, MC_SCHEM_string_view filename,
                                    const MC_SCHEM_save_option_litematica *option);

typedef struct {
  alignas(512) uint32_t compress_level;
  bool keep_air;
  //uint8_t reserved[507];
} MC_SCHEM_save_option_vanilla_structure;
static_assert(sizeof(MC_SCHEM_save_option_vanilla_structure)==512,"sizeof(MC_SCHEM_save_option_vanilla_structure) should be 512");
MC_SCHEM_EXPORT MC_SCHEM_save_option_vanilla_structure MC_SCHEM_save_option_vanilla_structure_default();
MC_SCHEM_EXPORT MC_SCHEM_error_box
MC_SCHEM_schem_save_vanilla_structure(const MC_SCHEM_schematic *, MC_SCHEM_writer writer,
                                      const MC_SCHEM_save_option_vanilla_structure *option);

MC_SCHEM_EXPORT MC_SCHEM_error_box
MC_SCHEM_schem_save_vanilla_structure_file(const MC_SCHEM_schematic *, MC_SCHEM_string_view filename,
                                           const MC_SCHEM_save_option_vanilla_structure *option);

typedef struct {
  alignas(512) uint32_t compress_level;
  MC_SCHEM_common_block background_block;
} MC_SCHEM_save_option_world_edit_13;
static_assert(sizeof(MC_SCHEM_save_option_world_edit_13)==512,"");
MC_SCHEM_EXPORT MC_SCHEM_save_option_world_edit_13 MC_SCHEM_save_option_world_edit_13_default();
MC_SCHEM_EXPORT MC_SCHEM_error_box
MC_SCHEM_schem_save_world_edit_13(const MC_SCHEM_schematic *, MC_SCHEM_writer writer,
                                  const MC_SCHEM_save_option_world_edit_13 *option);

MC_SCHEM_EXPORT MC_SCHEM_error_box
MC_SCHEM_schem_save_world_edit_13_file(const MC_SCHEM_schematic *, MC_SCHEM_string_view filename,
                                       const MC_SCHEM_save_option_world_edit_13 *option);

MC_SCHEM_DEFINE_OPTIONAL(int32_t, i32)
MC_SCHEM_DEFINE_OPTIONAL(int64_t, i64)
MC_SCHEM_DEFINE_OPTIONAL(MC_SCHEM_string_view, string_view)
MC_SCHEM_DEFINE_OPTIONAL(MC_SCHEM_array3_i32, i32_array3)

typedef struct {
  // universal
  alignas(1024) int32_t mc_data_version;
  int64_t time_created;
  int64_t time_modified;
  MC_SCHEM_string_view author;
  MC_SCHEM_string_view name;
  MC_SCHEM_string_view description;
  // litematica-related
  int32_t litematica_version;
  MC_SCHEM_optional_i32 litematica_subversion;

  // world edit 12&13 (.schem/.schematic) related
  int32_t schem_version;//world edit schem
  int32_t schem_offset[3];
  MC_SCHEM_optional_i32_array3 schem_we_offset;

  MC_SCHEM_optional_i64 date;

  //world edit 12 related
  MC_SCHEM_optional_string_view schem_world_edit_version;
  MC_SCHEM_optional_string_view schem_editing_platform;
  MC_SCHEM_optional_i32_array3 schem_origin;
  MC_SCHEM_string_view schem_material;//Classic or Alpha

} MC_SCHEM_schem_metadata_c_rep;
static_assert(sizeof(MC_SCHEM_schem_metadata_c_rep) == 1024, "sizeof(MC_SCHEM_schem_metadata_c_rep) should be 512");

MC_SCHEM_EXPORT MC_SCHEM_schem_metadata_c_rep MC_SCHEM_schem_get_metadata(const MC_SCHEM_schematic *);
MC_SCHEM_EXPORT void MC_SCHEM_schem_set_metadata(MC_SCHEM_schematic *, const MC_SCHEM_schem_metadata_c_rep *);

#ifdef __cplusplus
}
#endif

#endif  //MC_SCHEM_H