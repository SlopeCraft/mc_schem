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

size_t wrap_stream_read(void *handle, uint8_t *buffer, size_t buffer_size,
                        bool *ok, char *error, size_t error_capacity) {
  FILE *fp = (FILE *) handle;
  const size_t read_bytes = fread(buffer, 1, buffer_size, fp);
  *ok = true;
  return read_bytes;
}

MC_SCHEM_reader MC_SCHEM_reader_wrap_stream(FILE *fp) {
  MC_SCHEM_reader result;
  result.handle = fp;
  result.read_fun = wrap_stream_read;
  return result;
}