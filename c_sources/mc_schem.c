#include "mc_schem.h"

MC_SCHEM_string_view MC_SCHEM_c_string_to_string_view(const char *str) {
  const size_t len = strlen(str);
  MC_SCHEM_string_view result;
  result.begin = str;
  result.end = str + len;
  return result;
}

bool MC_SCHEM_map_contains_key(const MC_SCHEM_map_ref *map,
                               MC_SCHEM_map_key_type key_t,
                               const MC_SCHEM_key_wrapper *key) {
  bool ok = false;
  MC_SCHEM_value_wrapper find_result =
      MC_SCHEM_map_find(map, key_t, MC_SCHEM_map_get_value_type(map), key, &ok);
  if (!ok) {
    return false;
  }
  return (find_result.string != NULL);
}

bool MC_SCHEM_map_iterator_is_end(const MC_SCHEM_map_iterator *it) {
  return MC_SCHEM_map_iterator_length(it) == 0;
}