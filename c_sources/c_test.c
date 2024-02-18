#include <mc_schem.h>
#include <stdio.h>
#include <assert.h>
#include <stdlib.h>

void test_link();

void test_error();

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

  test_error();
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

  check_fun_ptr(MC_SCHEM_version_string);
  check_fun_ptr(MC_SCHEM_version_major);
  check_fun_ptr(MC_SCHEM_version_minor);
  check_fun_ptr(MC_SCHEM_version_patch);
  check_fun_ptr(MC_SCHEM_version_tweak);

  check_fun_ptr(MC_SCHEM_string_unwrap);
  check_fun_ptr(MC_SCHEM_c_string_to_string_view);
  check_fun_ptr(MC_SCHEM_string_set);


  check_fun_ptr(MC_SCHEM_map_unwrap_box);
  check_fun_ptr(MC_SCHEM_map_get_key_type);
  check_fun_ptr(MC_SCHEM_map_get_value_type);
  check_fun_ptr(MC_SCHEM_create_map);
  check_fun_ptr(MC_SCHEM_release_map);
  check_fun_ptr(MC_SCHEM_map_find);
  check_fun_ptr(MC_SCHEM_map_iterator_first);
  check_fun_ptr(MC_SCHEM_map_iterator_add);
  check_fun_ptr(MC_SCHEM_map_iterator_end);
  check_fun_ptr(MC_SCHEM_map_iterator_deref);
  check_fun_ptr(MC_SCHEM_map_iterator_length);
  check_fun_ptr(MC_SCHEM_map_iterator_is_end);
  check_fun_ptr(MC_SCHEM_map_iterator_equal);
  check_fun_ptr(MC_SCHEM_map_contains_key);
  check_fun_ptr(MC_SCHEM_map_length);
  check_fun_ptr(MC_SCHEM_map_capacity);
  check_fun_ptr(MC_SCHEM_map_reserve);
  check_fun_ptr(MC_SCHEM_map_foreach);
  check_fun_ptr(MC_SCHEM_map_insert);
  check_fun_ptr(MC_SCHEM_map_remove);
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
  ////////////////////////////////////
  //entity
  check_fun_ptr(MC_SCHEM_create_entity);
  check_fun_ptr(MC_SCHEM_release_block);
  check_fun_ptr(MC_SCHEM_entity_get_block_pos);
  check_fun_ptr(MC_SCHEM_entity_get_pos);
  check_fun_ptr(MC_SCHEM_entity_set_block_pos);
  check_fun_ptr(MC_SCHEM_entity_set_pos);
  check_fun_ptr(MC_SCHEM_entity_get_tags);
  ////////////////////////////////////
  //block entity
  check_fun_ptr(MC_SCHEM_create_block_entity);
  check_fun_ptr(MC_SCHEM_release_block_entity);
  check_fun_ptr(MC_SCHEM_block_entity_get_tags);
  ////////////////////////////////////
  //pending tick
  check_fun_ptr(MC_SCHEM_create_pending_tick);
  check_fun_ptr(MC_SCHEM_release_pending_tick);
  check_fun_ptr(MC_SCHEM_pending_tick_get_priority);
  check_fun_ptr(MC_SCHEM_pending_tick_set_priority);
  check_fun_ptr(MC_SCHEM_pending_tick_get_sub_tick);
  check_fun_ptr(MC_SCHEM_pending_tick_set_sub_tick);
  check_fun_ptr(MC_SCHEM_pending_tick_get_time);
  check_fun_ptr(MC_SCHEM_pending_tick_set_time);
  check_fun_ptr(MC_SCHEM_pending_tick_get_id);
  check_fun_ptr(MC_SCHEM_pending_tick_get_type);
  check_fun_ptr(MC_SCHEM_pending_tick_set_info);
  ////////////////////////////////////
  //error
  check_fun_ptr(MC_SCHEM_release_error);
  check_fun_ptr(MC_SCHEM_error_to_string);
  check_fun_ptr(MC_SCHEM_error_test_none);
  check_fun_ptr(MC_SCHEM_error_test_some);
  ////////////////////////////////////
  //region
  check_fun_ptr(MC_SCHEM_create_region);
  check_fun_ptr(MC_SCHEM_release_region);
  check_fun_ptr(MC_SCHEM_region_get_name);
  check_fun_ptr(MC_SCHEM_region_set_name);
  check_fun_ptr(MC_SCHEM_region_get_offset);
  check_fun_ptr(MC_SCHEM_region_set_offset);
  check_fun_ptr(MC_SCHEM_region_get_palette);
  check_fun_ptr(MC_SCHEM_region_set_palette);
  check_fun_ptr(MC_SCHEM_region_get_block_entities);
  check_fun_ptr(MC_SCHEM_region_get_pending_ticks);
  check_fun_ptr(MC_SCHEM_region_get_entities);
  check_fun_ptr(MC_SCHEM_region_get_block_index_array);
  check_fun_ptr(MC_SCHEM_region_get_number_id_array);
  check_fun_ptr(MC_SCHEM_region_get_shape);
  check_fun_ptr(MC_SCHEM_region_reshape);
  check_fun_ptr(MC_SCHEM_region_get_block);
  check_fun_ptr(MC_SCHEM_region_set_block);
  check_fun_ptr(MC_SCHEM_region_get_block_index);
  check_fun_ptr(MC_SCHEM_region_set_block_index);
  check_fun_ptr(MC_SCHEM_region_get_volume);
  check_fun_ptr(MC_SCHEM_region_get_total_blocks);
  check_fun_ptr(MC_SCHEM_region_get_block_index_of_air);
  check_fun_ptr(MC_SCHEM_region_get_block_index_of_structure_void);
  check_fun_ptr(MC_SCHEM_region_contains_coordinate);
  check_fun_ptr(MC_SCHEM_region_get_block_info);
  check_fun_ptr(MC_SCHEM_region_shrink_palette);

  ////////////////////////////////////
  //schem
  check_fun_ptr(MC_SCHEM_create_schem);
  check_fun_ptr(MC_SCHEM_release_schem);

  check_fun_ptr(MC_SCHEM_load_option_litematica_default);
  check_fun_ptr(MC_SCHEM_load_option_vanilla_structure_default);
  check_fun_ptr(MC_SCHEM_load_option_world_edit_13_default);
  check_fun_ptr(MC_SCHEM_load_option_world_edit_12_default);

  check_fun_ptr(MC_SCHEM_reader_wrap_stream);

  check_fun_ptr(MC_SCHEM_schem_load_litematica);
  check_fun_ptr(MC_SCHEM_schem_load_litematica_file);
  check_fun_ptr(MC_SCHEM_schem_load_litematica_bytes);
  check_fun_ptr(MC_SCHEM_schem_load_vanilla_structure);
  check_fun_ptr(MC_SCHEM_schem_load_vanilla_structure_file);
  check_fun_ptr(MC_SCHEM_schem_load_vanilla_structure_bytes);
  check_fun_ptr(MC_SCHEM_schem_load_world_edit_13);
  check_fun_ptr(MC_SCHEM_schem_load_world_edit_13_file);
  check_fun_ptr(MC_SCHEM_schem_load_world_edit_13_bytes);
  check_fun_ptr(MC_SCHEM_schem_load_world_edit_12);
  check_fun_ptr(MC_SCHEM_schem_load_world_edit_12_file);
  check_fun_ptr(MC_SCHEM_schem_load_world_edit_12_bytes);


  check_fun_ptr(MC_SCHEM_schem_save_litematica);
  check_fun_ptr(MC_SCHEM_schem_save_litematica_file);
  check_fun_ptr(MC_SCHEM_schem_save_vanilla_structure);
  check_fun_ptr(MC_SCHEM_schem_save_vanilla_structure_file);
  check_fun_ptr(MC_SCHEM_schem_save_world_edit_13);
  check_fun_ptr(MC_SCHEM_schem_save_world_edit_13_file);
//  check_fun_ptr();
//  check_fun_ptr();
//  check_fun_ptr();
//  check_fun_ptr();
//  check_fun_ptr();
//  check_fun_ptr();
//  check_fun_ptr();
//  check_fun_ptr();
//  check_fun_ptr();
//  check_fun_ptr();
//  check_fun_ptr();
//  check_fun_ptr();
//  check_fun_ptr();
//  check_fun_ptr();
//  check_fun_ptr();
//  check_fun_ptr();
}


void test_error() {
  MC_SCHEM_error_box box_none = MC_SCHEM_error_test_none();
  assert(box_none.ptr == NULL);
  MC_SCHEM_error_box box_some = MC_SCHEM_error_test_some();
  assert(box_some.ptr != NULL);
  char buf[1024];
  size_t length = 0;
  MC_SCHEM_error_to_string(box_some.ptr, buf, sizeof(buf), &length);
  if (length > 0) {
    printf("Error generated by MC_SCHEM_error_test_some is: \"%s\"\n", buf);
  }
  MC_SCHEM_release_error(&box_some);
}