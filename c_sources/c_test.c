#include <mc_schem.h>
#include <stdio.h>
#include <assert.h>
#include <stdlib.h>

void test_link();

int main(int argc, char **argv) {
  printf("version of mc_schem: %s\n", MC_SCHEM_version_string());

  test_link();

  {
    MC_SCHEM_block_box block = MC_SCHEM_create_block();
    assert(block.ptr);
    MC_SCHEM_block_id_parse_error error;
    bool ok = MC_SCHEM_parse_block(
      MC_SCHEM_c_string_to_string_view("minecraft:air"),
      block.ptr,
      &error);
    assert(ok);
    MC_SCHEM_release_block(&block);
  }
//  MC_SCHEM_schematic schem = MC_SCHEM_create_schem();
//  MC_SCHEM_destroy_schem(&schem);
  return 0;
}

void check_fun_ptr(void *fun_addr) {
  if (fun_addr == NULL) {
    abort();
  }
}
void test_link() {

  check_fun_ptr(MC_SCHEM_string_unwrap);
  check_fun_ptr(MC_SCHEM_c_string_to_string_view);

  check_fun_ptr(MC_SCHEM_version_string);
  check_fun_ptr(MC_SCHEM_version_major);
  check_fun_ptr(MC_SCHEM_version_minor);
  check_fun_ptr(MC_SCHEM_version_patch);
  check_fun_ptr(MC_SCHEM_version_tweak);

  check_fun_ptr(MC_SCHEM_map_unwrap_box);
  check_fun_ptr(MC_SCHEM_map_get_key_type);
  check_fun_ptr(MC_SCHEM_map_get_value_type);
  check_fun_ptr(MC_SCHEM_create_map);
  check_fun_ptr(MC_SCHEM_release_map);
  check_fun_ptr(MC_SCHEM_map_find);
  check_fun_ptr(MC_SCHEM_map_iterator_first);
  check_fun_ptr(MC_SCHEM_map_iterator_add);
  check_fun_ptr(MC_SCHEM_map_iterator_deref);
  check_fun_ptr(MC_SCHEM_map_iterator_length);
  check_fun_ptr(MC_SCHEM_map_iterator_is_end);
  check_fun_ptr(MC_SCHEM_map_contains_key);
  check_fun_ptr(MC_SCHEM_map_length);
  check_fun_ptr(MC_SCHEM_map_capacity);
  check_fun_ptr(MC_SCHEM_map_reserve);
  //////////
  check_fun_ptr(MC_SCHEM_create_nbt);
  check_fun_ptr(MC_SCHEM_release_nbt);

  check_fun_ptr(MC_SCHEM_nbt_get_type);
  check_fun_ptr(MC_SCHEM_nbt_get_byte);
  check_fun_ptr(MC_SCHEM_nbt_get_short);
  check_fun_ptr(MC_SCHEM_nbt_get_int);
  check_fun_ptr(MC_SCHEM_nbt_get_long);
  check_fun_ptr(MC_SCHEM_nbt_get_float);
  check_fun_ptr(MC_SCHEM_nbt_get_double);
  check_fun_ptr(MC_SCHEM_nbt_get_string);
  check_fun_ptr(MC_SCHEM_nbt_get_byte_array);
  check_fun_ptr(MC_SCHEM_nbt_get_int_array);
  check_fun_ptr(MC_SCHEM_nbt_get_long_array);
  check_fun_ptr(MC_SCHEM_nbt_get_list);
  check_fun_ptr(MC_SCHEM_nbt_set_byte);
  check_fun_ptr(MC_SCHEM_nbt_set_short);
  check_fun_ptr(MC_SCHEM_nbt_set_int);
  check_fun_ptr(MC_SCHEM_nbt_set_long);
  check_fun_ptr(MC_SCHEM_nbt_set_float);
  check_fun_ptr(MC_SCHEM_nbt_set_double);
  check_fun_ptr(MC_SCHEM_nbt_set_string);
  check_fun_ptr(MC_SCHEM_nbt_set_byte_array);
  check_fun_ptr(MC_SCHEM_nbt_set_int_array);
  check_fun_ptr(MC_SCHEM_nbt_set_long_array);
  check_fun_ptr(MC_SCHEM_nbt_set_list);
  check_fun_ptr(MC_SCHEM_nbt_get_compound);
  check_fun_ptr(MC_SCHEM_nbt_set_compound);
  ////////////////////////////////////
  //blocks
  check_fun_ptr(MC_SCHEM_create_block);
  check_fun_ptr(MC_SCHEM_release_block);
  check_fun_ptr(MC_SCHEM_block_get_namespace);
  check_fun_ptr(MC_SCHEM_block_get_id);
  check_fun_ptr(MC_SCHEM_block_get_attributes);
  check_fun_ptr(MC_SCHEM_block_set_namespace);
  check_fun_ptr(MC_SCHEM_block_set_id);
  check_fun_ptr(MC_SCHEM_block_set_attributes);
  check_fun_ptr(MC_SCHEM_parse_block);
  check_fun_ptr(MC_SCHEM_block_to_full_id);
  //  check_fun_ptr(MC_SCHEM_destroy_schem);
}