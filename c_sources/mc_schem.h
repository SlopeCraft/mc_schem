//
// Created by Joseph on 2024/1/17.
//

#ifndef MC_SCHEM_MC_SCHEM_H
#define MC_SCHEM_MC_SCHEM_H

#include <stdint.h>
#include <mc_schem_export.h>

#ifdef __cplusplus
extern "C"{
#endif

//inline uint64_t MC_SCHEM_make_version(uint16_t major, uint16_t minor, uint16_t patch, uint16_t tweak) {
//  return (((uint64_t) major) << 24) | (((uint64_t) minor) << 16) | (((uint64_t) patch) << 8) |
//         (((uint64_t) tweak) << 0);
//}


MC_SCHEM_EXPORT const char *MC_SCHEM_version_string();

struct MC_SCHEM_rust_str {
  const char *begin;
  const char *end;
};

struct MC_SCHEM_nbt_value;


struct MC_SCHEM_entity;
struct MC_SCHEM_block;
struct MC_SCHEM_block_entity;
struct MC_SCHEM_pending_tick;
struct MC_SCHEM_region;

typedef struct {
  void *reserved;
} MC_SCHEM_schematic;

MC_SCHEM_EXPORT MC_SCHEM_schematic MC_SCHEM_create_schem();

MC_SCHEM_EXPORT void MC_SCHEM_destroy_schem(MC_SCHEM_schematic *);


#ifdef __cplusplus
};
#endif

#endif //MC_SCHEM_MC_SCHEM_H
