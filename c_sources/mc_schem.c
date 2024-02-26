/*
mc_schem is a rust library to generate, load, manipulate and save minecraft
schematic files. Copyright (C) 2024  joseph

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

size_t
wrap_stream_write(void *handle, const uint8_t *buf, size_t buf_size, bool *ok, char *error, size_t error_capacity) {
  FILE *fp = (FILE *) handle;
  const size_t write_bytes = fwrite(buf, 1, buf_size, fp);
  *ok = true;
  return write_bytes;
}

void wrap_stream_flush(void *handle, bool *ok, char *error, size_t error_capacity) {
  FILE *fp = (FILE *) handle;
  const int error_code = fflush(fp);
  if (error_code != 0) {
    *ok = false;
    snprintf(error, error_capacity, "Failed to flush c FILE stream, function fflush returned error code %d",
             error_code);
  } else {
    *ok = true;
  }
}

MC_SCHEM_writer MC_SCHEM_writer_wrap_stream(FILE *fp) {
  MC_SCHEM_writer result;
  result.handle = fp;
  result.write_fun = wrap_stream_write;
  result.flush_fun = wrap_stream_flush;
  return result;
}