#include <mc_schem.h>
#include <stdio.h>
#include <assert.h>

void test_link();

int main(int argc, char **argv) {
  printf("version of mc_schem: %s\n", MC_SCHEM_version_string());

  test_link();
  MC_SCHEM_schematic schem = MC_SCHEM_create_schem();
  MC_SCHEM_destroy_schem(&schem);
  return 0;
}

void test_link() {
  assert(MC_SCHEM_version_string);
  assert(MC_SCHEM_rust_object_get_null);
  assert(MC_SCHEM_rust_object_manual_init);
  assert(MC_SCHEM_rust_object_is_reference);
  assert(MC_SCHEM_rust_object_is_null);
  assert(MC_SCHEM_map_get_key_type);
  assert(MC_SCHEM_map_get_value_type);
  assert(MC_SCHEM_map_get_size);
  assert(MC_SCHEM_map_find_const);
  assert(MC_SCHEM_map_find_mut);
  assert(MC_SCHEM_map_contains_key);
  assert(MC_SCHEM_map_insert);
  assert(MC_SCHEM_nbt_create_scalar);
  assert(MC_SCHEM_nbt_release_value);
  assert(MC_SCHEM_nbt_get_type);
  assert(MC_SCHEM_nbt_is_null);
  assert(MC_SCHEM_nbt_is_reference);
  assert(MC_SCHEM_nbt_get_scalar);
  assert(MC_SCHEM_nbt_get_length);
  assert(MC_SCHEM_nbt_get_list_element_const);
  assert(MC_SCHEM_nbt_get_list_element_mut);
  assert(MC_SCHEM_nbt_get_string);
  assert(MC_SCHEM_nbt_get_scalar_array_const);
  assert(MC_SCHEM_nbt_get_scalar_array_mut);
  assert(MC_SCHEM_nbt_get_compound_const);
  assert(MC_SCHEM_nbt_get_compound_mut);
//  assert();
//  assert();
//  assert();
//  assert();
//  assert();
//  assert();
//  assert();
//  assert();
//  assert();
//  assert();
  assert(MC_SCHEM_create_schem);
  assert(MC_SCHEM_destroy_schem);
}