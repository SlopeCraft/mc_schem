#ifndef MC_SCHEM_H
#define MC_SCHEM_H

#include <mc_schem_export.h>
#include <stdint.h>
#include <stdbool.h>
#include <stddef.h>
#include <string.h>

#define MC_SCHEM_DEFINE_BOX(content_type) \
typedef struct {                          \
  content_type*ptr;                       \
} content_type##_box;

#ifdef __cplusplus
extern "C" {};
#endif


MC_SCHEM_EXPORT const char *MC_SCHEM_version_string();

MC_SCHEM_EXPORT uint16_t MC_SCHEM_version_major();

MC_SCHEM_EXPORT uint16_t MC_SCHEM_version_minor();

MC_SCHEM_EXPORT uint16_t MC_SCHEM_version_patch();

MC_SCHEM_EXPORT uint16_t MC_SCHEM_version_tweak();

/////////////////////////////////////////////

typedef struct {
  const char *begin;
  const char *end;
} MC_SCHEM_string_view;

typedef struct MC_SCHEM_string_s MC_SCHEM_string;

MC_SCHEM_EXPORT MC_SCHEM_string_view MC_SCHEM_string_unwrap(const MC_SCHEM_string *);

inline MC_SCHEM_string_view MC_SCHEM_c_string_to_string_view(const char *str) {
  const size_t len = strlen(str);
  MC_SCHEM_string_view result;
  result.begin = str;
  result.end = str + len;
  return result;
}

//////////////////////////////////

typedef struct {
  size_t reserved[7];
} MC_SCHEM_nbt_value;
typedef struct MC_SCHEM_block_s MC_SCHEM_block;
typedef struct MC_SCHEM_block_entity_s MC_SCHEM_block_entity;
typedef struct MC_SCHEM_pending_tick_s MC_SCHEM_pending_tick;

typedef enum : uint8_t {
  MC_SCHEM_MKT_string,
  MC_SCHEM_MKT_pos_i32,
} MC_SCHEM_map_key_type;

typedef enum : uint8_t {
  MC_SCHEM_MVT_string,
  MC_SCHEM_MVT_nbt,
  MC_SCHEM_MVT_block_entity,
  MC_SCHEM_MVT_pending_tick,
} MC_SCHEM_map_value_type;

typedef struct {
  size_t reserved[2];
} MC_SCHEM_map_ref;//typed pointer to a BTreeMap/HashMap

typedef struct {
  size_t reserved[2];
} MC_SCHEM_map_box;

typedef struct {
  size_t reserved[10];
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
MC_SCHEM_map_find(const MC_SCHEM_map_ref *map, MC_SCHEM_map_key_type key_t, MC_SCHEM_map_value_type val_t,
                  const MC_SCHEM_key_wrapper *key, bool *ok);

inline bool
MC_SCHEM_map_contains_key(const MC_SCHEM_map_ref *map, MC_SCHEM_map_key_type key_t, const MC_SCHEM_key_wrapper *key) {
  bool ok = false;
  MC_SCHEM_value_wrapper find_result = MC_SCHEM_map_find(map, key_t, MC_SCHEM_map_get_value_type(map), key, &ok);
  if (!ok) {
    return false;
  }
  return (find_result.string != NULL);
}

MC_SCHEM_EXPORT size_t MC_SCHEM_map_length(const MC_SCHEM_map_ref *map);

MC_SCHEM_EXPORT void MC_SCHEM_map_reserve(MC_SCHEM_map_ref *map, size_t new_capacity);

MC_SCHEM_EXPORT MC_SCHEM_map_iterator
MC_SCHEM_map_iterator_first(const MC_SCHEM_map_ref *map, MC_SCHEM_map_key_type key_t, MC_SCHEM_map_value_type val_t,
                            bool *ok);

MC_SCHEM_EXPORT struct {
  MC_SCHEM_key_wrapper key;
  MC_SCHEM_value_wrapper value;
  bool has_value;
} MC_SCHEM_map_iterator_next(MC_SCHEM_map_iterator *it);

MC_SCHEM_EXPORT size_t MC_SCHEM_map_iterator_length(const MC_SCHEM_map_iterator *it);

inline bool MC_SCHEM_map_iterator_is_end(const MC_SCHEM_map_iterator *it) {
  return MC_SCHEM_map_iterator_length(it) == 0;
}

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

MC_SCHEM_EXPORT MC_SCHEM_nbt_value_box MC_SCHEM_nbt_create();

MC_SCHEM_EXPORT void MC_SCHEM_nbt_release(MC_SCHEM_nbt_value_box *nbt_box);

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

MC_SCHEM_EXPORT MC_SCHEM_string MC_SCHEM_nbt_get_string(const MC_SCHEM_nbt_value *, bool *ok);

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
typedef struct MC_SCHEM_block_s MC_SCHEM_block;
MC_SCHEM_DEFINE_BOX(MC_SCHEM_block)

MC_SCHEM_EXPORT MC_SCHEM_block_box MC_SCHEM_create_block();

MC_SCHEM_EXPORT void MC_SCHEM_release_block(MC_SCHEM_map_box *);

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

typedef struct MC_SCHEM_entity_s MC_SCHEM_entity;
MC_SCHEM_DEFINE_BOX(MC_SCHEM_entity)
typedef struct MC_SCHEM_region_s MC_SCHEM_region;
MC_SCHEM_DEFINE_BOX(MC_SCHEM_region)
typedef struct MC_SCHEM_schem_s MC_SCHEM_schem;
MC_SCHEM_DEFINE_BOX(MC_SCHEM_schem)

#ifdef __cplusplus
}
#endif

#endif  //MC_SCHEM_H