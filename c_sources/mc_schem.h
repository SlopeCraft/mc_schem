//
// Created by Joseph on 2024/1/17.
//

#ifndef MC_SCHEM_MC_SCHEM_H
#define MC_SCHEM_MC_SCHEM_H
#ifdef __cplusplus

#include <cstdbool>
#include <cstddef>
#include <cstdint>

#else
#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>
#endif
#include <mc_schem_export.h>

#ifdef __cplusplus
extern "C"{
#endif

//inline uint64_t MC_SCHEM_make_version(uint16_t major, uint16_t minor, uint16_t patch, uint16_t tweak) {
//  return (((uint64_t) major) << 24) | (((uint64_t) minor) << 16) | (((uint64_t) patch) << 8) |
//         (((uint64_t) tweak) << 0);
//}


MC_SCHEM_EXPORT const char *MC_SCHEM_version_string();

typedef struct {
  const char *begin;
  const char *end;
} MC_SCHEM_rust_str;


typedef struct {
  size_t reserved[2];
} MC_SCHEM_rust_object;

MC_SCHEM_EXPORT MC_SCHEM_rust_object MC_SCHEM_rust_object_get_null();

inline void MC_SCHEM_rust_object_manual_init(MC_SCHEM_rust_object *ro) {
  *ro = MC_SCHEM_rust_object_get_null();
}

MC_SCHEM_EXPORT bool MC_SCHEM_rust_object_is_reference(const MC_SCHEM_rust_object *obj);
MC_SCHEM_EXPORT bool MC_SCHEM_rust_object_is_null(const MC_SCHEM_rust_object *obj);

//------------------------------------map
// reference---------------------------------------
typedef enum {
  MC_SCHEM_map_key_string, // MC_SCHEM_rust_str
  MC_SCHEM_map_pos_i32,    // int32_t[3]
} MC_SCHEM_map_key_type;

typedef enum {
  MC_SCHEM_map_value_string,
  MC_SCHEM_map_value_nbt,
  MC_SCHEM_map_value_block_entity,
  MC_SCHEM_map_value_pending_tick,
} MC_SCHEM_map_value_type;

// reference to a map/hashmap
typedef struct {
  size_t reserved[2];
} MC_SCHEM_map_ref;

MC_SCHEM_EXPORT MC_SCHEM_map_key_type
MC_SCHEM_map_get_key_type(const MC_SCHEM_map_ref *);

MC_SCHEM_EXPORT MC_SCHEM_map_value_type
MC_SCHEM_map_get_value_type(const MC_SCHEM_map_ref *);

MC_SCHEM_EXPORT size_t MC_SCHEM_map_get_size(const MC_SCHEM_map_ref *);

MC_SCHEM_EXPORT bool MC_SCHEM_map_find_const(const MC_SCHEM_map_ref *,
                                             MC_SCHEM_map_key_type,
                                             const void *key, void *value);

MC_SCHEM_EXPORT bool MC_SCHEM_map_find_mut(MC_SCHEM_map_ref *,
                                           MC_SCHEM_map_key_type,
                                           const void *key, void *value);

MC_SCHEM_EXPORT bool MC_SCHEM_map_contains_key(const MC_SCHEM_map_ref *map,
                                               MC_SCHEM_map_key_type t,
                                               const void *key);

MC_SCHEM_EXPORT bool MC_SCHEM_map_insert(MC_SCHEM_map_ref *,
                                         MC_SCHEM_map_key_type, const void *key,
                                         const void *value);

//------------------------------------nbt wrappers---------------------------------------
typedef struct {
  MC_SCHEM_rust_object reserved;
} MC_SCHEM_nbt_value;

typedef enum MC_SCHEM_nbt_type_e {
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


typedef struct {
  MC_SCHEM_rust_object reserved;
} MC_SCHEM_nbt_compound;

typedef struct {
  MC_SCHEM_rust_object reserved;
} MC_SCHEM_nbt_list;

MC_SCHEM_EXPORT MC_SCHEM_nbt_value MC_SCHEM_nbt_create_scalar(
    MC_SCHEM_nbt_type type, const void *value, bool *success_nullable);

MC_SCHEM_EXPORT void MC_SCHEM_nbt_release_value(MC_SCHEM_nbt_value *value);

MC_SCHEM_EXPORT MC_SCHEM_nbt_type MC_SCHEM_nbt_get_type(const MC_SCHEM_nbt_value *value);

inline bool MC_SCHEM_nbt_is_null(const MC_SCHEM_nbt_value *value) {
  return MC_SCHEM_rust_object_is_null(&value->reserved);
}

inline bool MC_SCHEM_nbt_is_reference(const MC_SCHEM_nbt_value *value) {
  return MC_SCHEM_rust_object_is_reference(&value->reserved);
}

MC_SCHEM_EXPORT bool MC_SCHEM_nbt_get_scalar(const MC_SCHEM_nbt_value *value, void *dest, size_t dest_capacity);

// return actual length for byte array, string, compound, list, int array and long array, ret urn 0 for scalar
MC_SCHEM_EXPORT bool MC_SCHEM_nbt_get_length(const MC_SCHEM_nbt_value *value, size_t *length);

// returns MC_SCHEM_nbt_value which will always contain a reference
MC_SCHEM_EXPORT const MC_SCHEM_nbt_value
MC_SCHEM_nbt_get_list_element_const(const MC_SCHEM_nbt_value *list, size_t index);

// returns MC_SCHEM_nbt_value which will always contain a reference
MC_SCHEM_EXPORT MC_SCHEM_nbt_value
MC_SCHEM_nbt_get_list_element_mut(MC_SCHEM_nbt_value *list, size_t index);

MC_SCHEM_EXPORT bool MC_SCHEM_nbt_get_string(const MC_SCHEM_nbt_value *tag_string, MC_SCHEM_rust_str *dest);

// for byte array, int array, long array
MC_SCHEM_EXPORT bool
MC_SCHEM_nbt_get_scalar_array_const(const MC_SCHEM_nbt_value *tag,
                                    const void **dest_ptr_nullable,
                                    size_t *num_elements_nullable);

// for byte array, int array, long array
MC_SCHEM_EXPORT bool
MC_SCHEM_nbt_get_scalar_array_mut(MC_SCHEM_nbt_value *tag,
                                  void **dest_ptr_nullable,
                                  size_t *num_elements_nullable);

MC_SCHEM_EXPORT const MC_SCHEM_map_ref MC_SCHEM_nbt_get_compound_const(
    const MC_SCHEM_nbt_value *tag, bool *ok_nullable);

MC_SCHEM_EXPORT MC_SCHEM_map_ref
MC_SCHEM_nbt_get_compound_mut(MC_SCHEM_nbt_value *tag, bool *ok_nullable);
//------------------------------------regions---------------------------------------
typedef struct {
  MC_SCHEM_rust_object reserved;
} MC_SCHEM_entity;
typedef struct {
  MC_SCHEM_rust_object reserved;
} MC_SCHEM_block;
typedef struct {
  MC_SCHEM_rust_object reserved;
} MC_SCHEM_block_entity;
typedef struct {
  MC_SCHEM_rust_object reserved;
} MC_SCHEM_pending_tick;
typedef struct {
  MC_SCHEM_rust_object reserved;
} MC_SCHEM_region;

//------------------------------------schematics---------------------------------------
typedef struct {
  MC_SCHEM_rust_object reserved;
} MC_SCHEM_schematic;

MC_SCHEM_EXPORT MC_SCHEM_schematic MC_SCHEM_create_schem();

MC_SCHEM_EXPORT void MC_SCHEM_destroy_schem(MC_SCHEM_schematic *);


#ifdef __cplusplus
}
#endif

#endif //MC_SCHEM_MC_SCHEM_H
